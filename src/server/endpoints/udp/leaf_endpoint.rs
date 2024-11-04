use std::net::SocketAddr;
use std::sync::Arc;

use bytes::BytesMut;
use log::{debug, error};
use tokio::io::{Error, ErrorKind, Result};
use tokio::net::UdpSocket;
use tokio::sync::mpsc::channel;

use crate::common::channel_socket::{ChannelPacket, ChannelSocket};
use crate::common::connection::Connection;
use crate::common::data_bridge::UdpSession;
use crate::server::endpoints::udp::client_host::Client;
use crate::server::endpoints::udp::messages::ClientConnect;

use super::udp_services::UdpServices;

pub async fn start(port: u16, services: Arc<UdpServices>) -> Result<()> {
    let config = services.get_config();
    let cancel_token = services.get_cancel_token();
    let hub_tx = services.get_leaf_hub_tx();

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

    let (leaf_data_tx, mut leaf_data_rx) = channel::<ChannelPacket>(100);

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

                {
                    let mut client_host = services.get_client_host().await;
                    if client_host.client_exists(&address) {
                        client_host.send(&address, data).await;
                        continue;
                    }
                }

                debug!(
                    "Accepted UDP connection from client '{}' at port {}",
                    address, port
                );

                let cancel_token = cancel_token.child_token();

                let channel_socket = ChannelSocket::new(leaf_data_tx.clone(), cancel_token.clone());

                services.get_client_host().await.add(Client::new(channel_socket.get_id(), port, address, channel_socket.get_socket_tx(), cancel_token.clone()));

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
                    Some(ChannelPacket(client_id, data)) => {
                        if let Some(client_address) = services.get_client_host().await.get_client_address(&client_id) {
                            if let Err(e) = connection.write_all_to(&data, &client_address).await {
                                debug!("Failed to send data to client. Reason: {}", e);
                                continue;
                            };
                            services.get_client_host().await.update_activity(&client_id);
                        } else {
                            error!("No target address set, cannot send UDP datagram at port '{}'.", port);
                        }
                    }
                    None => {
                        cancel_token.cancel();
                        continue;
                    }
                }
            },
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
