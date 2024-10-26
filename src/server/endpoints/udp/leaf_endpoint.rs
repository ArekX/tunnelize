use std::net::SocketAddr;
use std::sync::Arc;

use log::{debug, error};
use tokio::io::{Error, ErrorKind, Result};
use tokio::net::UdpSocket;

use crate::common::channel::RequestSender;
use crate::common::channel_socket::ChannelSocket;
use crate::common::connection::ConnectionStream;
use crate::server::endpoints::udp::messages::ClientConnect;
use crate::server::services::Services;

use super::configuration::UdpEndpointConfig;
use super::messages::UdpChannelRequest;

pub async fn start(
    port: u16,
    hub_tx: RequestSender<UdpChannelRequest>,
    config: Arc<UdpEndpointConfig>,
    services: Arc<Services>,
) -> Result<()> {
    let cancel_token = services.get_cancel_token();

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

                debug!(
                    "Accepted UDP connection from client '{}' at port {}",
                    address, port
                );

                let Ok(_) = hub_tx
                    .request(ClientConnect {
                        initial_data: Some(data),
                        stream: Some(ConnectionStream::from(ChannelSocket::new(leaf_data_tx.clone()))),
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
                if let Some(data) = data {
                    // TODO: This part probably won't work as expected, we dont know address of the client yet
                    // we need to keep the proper address so that we know where we are sending the data to
                    // and to reject any other data that comes from different address
                    let Ok(_) = connection.write_all(&data).await else {
                        error!("Failed to send data to client.");
                        continue;
                    };
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
