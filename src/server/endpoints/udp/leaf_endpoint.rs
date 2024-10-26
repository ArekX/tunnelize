use std::net::SocketAddr;
use std::sync::Arc;

use log::{debug, error};
use tokio::io::{Error, ErrorKind, Result};
use tokio::net::UdpSocket;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

use crate::common::channel::RequestSender;
use crate::common::channel_socket::ChannelSocket;
use crate::common::connection::ConnectionStream;
use crate::server::endpoints::udp::messages::ClientConnect;
use crate::server::services::Services;

use super::client_host::ClientHost;
use super::configuration::UdpEndpointConfig;
use super::messages::UdpChannelRequest;

pub async fn start(
    port: u16,
    hub_tx: RequestSender<UdpChannelRequest>,
    config: Arc<UdpEndpointConfig>,
    client_host: Arc<Mutex<ClientHost>>,
    services: Arc<Services>,
) -> Result<()> {
    let cancel_token = services.get_cancel_token();

    let mut target_address: Option<SocketAddr> = None;
    let mut socket_tx: Option<Sender<Vec<u8>>> = None;

    // TODO: each leaf has to have its own channel
    // TODO: on first connect add to client host, on cleanup tell the leaf_tx that client is gone

    let mut connection = match UdpSocket::bind(config.get_bind_address(port)).await {
        Ok(socket) => ConnectionStream::from(socket),
        Err(e) => {
            error!("Failed to bind client listener: {}", e);
            return Err(Error::new(
                ErrorKind::Other,
                "Failed to bind client listener",
            ));
        }
    };

    let (leaf_data_tx, mut leaf_data_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(1);

    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                debug!("Leaf endpoint for port '{}' cancelled.", port);
                break;
            }
            result = wait_for_client(&mut connection) => {
                 let Ok((data, address)) = result else {
                    error!("Failed to accept connection.");
                    continue;
                };

                if let Some(check_addr) = target_address {
                    if check_addr != address {
                        error!("Received UDP datagram from other address '{}' while assigned to address '{}'.", address, check_addr);
                        continue;
                    }

                    let Some(socket_tx) = &socket_tx else {
                        error!("No socket to send data to client.");
                        continue;
                    };

                    if let Err(e) = socket_tx.send(data).await {
                        error!("Failed to send data to client. Reason: {}", e);
                    }

                    continue;
                }

                debug!(
                    "Accepted UDP connection from client '{}' at port {}",
                    address, port
                );

                target_address = Some(address);

                let cleanup_token = cancel_token.child_token();

                let channel_socket = ChannelSocket::new(leaf_data_tx.clone(), cleanup_token.clone());

                socket_tx = Some(channel_socket.get_socket_tx());

                // TODO: Add cleanup for the channel socket

                let Ok(_) = hub_tx
                    .request(ClientConnect {
                        initial_data: Some(data),
                        stream: Some(ConnectionStream::from(channel_socket)),
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
                        if let Some(address) = target_address {
                            if let Err(e) = connection.write_all_to(&data, &address).await {
                                debug!("Failed to send data to client. Reason: {}", e);
                                continue;
                            };
                        } else {
                            error!("No target address set, cannot send UDP datagram at port '{}'.", port);
                        }
                    }
                    None => {
                        target_address = None;
                        continue;
                    }
                }
            }
        }
    }

    Ok(())
}

async fn wait_for_client(connection: &mut ConnectionStream) -> Result<(Vec<u8>, SocketAddr)> {
    let mut buffer = vec![0u8; 65537];

    let Ok((size, address)) = connection.read_with_address(&mut buffer).await else {
        return Err(Error::new(ErrorKind::Other, "Failed to receive data"));
    };

    debug!("Received {} bytes from '{}'", size, address);

    Ok((buffer[..size].to_vec(), address))
}
