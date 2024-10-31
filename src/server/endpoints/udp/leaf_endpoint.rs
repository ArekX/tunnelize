use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use bytes::BytesMut;
use log::{debug, error};
use tokio::io::{Error, ErrorKind, Result};
use tokio::net::UdpSocket;
use tokio::sync::mpsc::{channel, Sender};
use tokio::sync::Mutex;
use tokio::time::interval;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::common::channel::RequestSender;
use crate::common::channel_socket::ChannelSocket;
use crate::common::connection::Connection;
use crate::common::data_bridge::UdpSession;
use crate::server::endpoints::udp::messages::ClientConnect;
use crate::server::services::Services;

use super::activity_tracker::ActivityTracker;
use super::configuration::UdpEndpointConfig;
use super::messages::UdpChannelRequest;

pub struct TargetClient {
    id: Uuid,
    address: SocketAddr,
    socket_tx: Sender<Vec<u8>>,
    cancel_token: CancellationToken,
}

pub async fn start(
    port: u16,
    hub_tx: RequestSender<UdpChannelRequest>,
    config: Arc<UdpEndpointConfig>,
    activity_tracker: Arc<Mutex<ActivityTracker>>,
    services: Arc<Services>,
) -> Result<()> {
    let cancel_token = services.get_cancel_token();

    let mut target_client: Option<TargetClient> = None;

    let mut connection = match UdpSocket::bind(config.get_bind_address(port)).await {
        Ok(socket) => Connection::from(socket),
        Err(e) => {
            error!("Failed to bind client listener: {}", e);
            return Err(Error::new(
                ErrorKind::Other,
                "Failed to bind client listener",
            ));
        }
    };

    let (leaf_data_tx, mut leaf_data_rx) = channel::<Vec<u8>>(1);

    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                debug!("Leaf endpoint for UDP port '{}' cancelled.", port);
                break;
            }
            result = wait_for_client(&mut connection) => {
                 let Ok((data, address)) = result else {
                    error!("Failed to accept connection.");
                    continue;
                };

                if let Some(client) = target_client.as_ref() {
                    // TODO: this probably wont work either as there can be many clients, probably need client tracking for multiple clients
                    if client.address == address {
                        activity_tracker.lock().await.update_activity(&client.id);
                        if let Err(e) = client.socket_tx.send(data).await {
                            error!("Failed to send data to client. Reason: {:?}", e);
                            target_client.take();
                            // TODO: Cancel client
                        }

                        continue;
                    }
                }

                debug!(
                    "Accepted UDP connection from client '{}' at port {}",
                    address, port
                );

                let cancel_token = cancel_token.child_token();

                let channel_socket = ChannelSocket::new(leaf_data_tx.clone(), cancel_token.clone());

                let id = activity_tracker.lock().await.add(cancel_token.clone());

                target_client = Some(TargetClient {
                    id,
                    address,
                    socket_tx: channel_socket.get_socket_tx(),
                    cancel_token: cancel_token.clone(),
                });

                let Ok(_) = hub_tx
                    .request(ClientConnect {
                        initial_data: Some(data),
                        stream: Some(Connection::from(channel_socket)),
                        session: Some(UdpSession {
                            address,
                            cancel_token
                        }),
                        port,
                    })
                    .await
                else {
                    error!("Failed to send leaf connection request.");
                    continue;
                };

                debug!("Sent leaf connection request.");
            },
            data = leaf_data_rx.recv() => {
                match data {
                    Some(data) => {
                        if let Some(client) = target_client.as_ref() {
                            activity_tracker.lock().await.update_activity(&client.id);
                            if let Err(e) = connection.write_all_to(&data, &client.address).await {
                                debug!("Failed to send data to client. Reason: {}", e);
                                continue;
                            };
                        } else {
                            error!("No target address set, cannot send UDP datagram at port '{}'.", port);
                        }
                    }
                    None => {
                        if let Some(client) = target_client.take() {
                            debug!("Target client cancelled, removing from activity tracker.");
                            activity_tracker.lock().await.cancel(&client.id);
                        }

                        continue;
                    }
                }
            },
            _ = track_target_client(&mut target_client) => {}
        }
    }

    Ok(())
}

async fn track_target_client(target_client: &mut Option<TargetClient>) -> Result<()> {
    let mut interval = interval(Duration::from_secs(30));

    loop {
        interval.tick().await;

        if let Some(client) = target_client.as_ref() {
            client.cancel_token.cancelled().await;
            target_client.take();
            debug!("Target client cancelled.");
            break;
        }
    }

    Ok(())
}

async fn wait_for_client(connection: &mut Connection) -> Result<(Vec<u8>, SocketAddr)> {
    let mut buffer = BytesMut::with_capacity(2048);
    buffer.resize(2048, 0);

    match connection.read_with_address(&mut buffer).await {
        Ok((size, address)) => Ok((buffer[..size].to_vec(), address)),
        Err(e) => {
            error!("Failed to read data from client: {:?}", e);
            Err(e)
        }
    }
}
