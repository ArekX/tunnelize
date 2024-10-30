use std::sync::Arc;

use log::{debug, error};
use tokio::io::{Error, ErrorKind, Result};
use tokio::net::TcpListener;

use crate::common::channel::RequestSender;
use crate::common::connection::Connection;
use crate::server::services::Services;

use super::configuration::TcpEndpointConfig;
use super::messages::{ClientConnect, TcpChannelRequest};

pub async fn start(
    port: u16,
    hub_tx: RequestSender<TcpChannelRequest>,
    config: Arc<TcpEndpointConfig>,
    services: Arc<Services>,
) -> Result<()> {
    let cancel_token = services.get_cancel_token();

    let listener = match TcpListener::bind(config.get_bind_address(port)).await {
        Ok(listener) => listener,
        Err(e) => {
            error!("Failed to bind client listener: {}", e);
            return Err(Error::new(
                ErrorKind::Other,
                "Failed to bind client listener",
            ));
        }
    };

    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                debug!("Leaf endpoint for TCP port '{}' cancelled.", port);
                break;
            }
            result = listener.accept() => {
                 let Ok((stream, address)) = result else {
                    error!("Failed to accept connection.");
                    continue;
                };

                debug!(
                    "Accepted TCP connection from client '{}' at port {}",
                    address, port
                );

                let Ok(_) = hub_tx
                    .request(ClientConnect {
                        stream: Some(Connection::from(stream)),
                        port,
                    })
                    .await
                else {
                    error!("Failed to send leaf connection request.");
                    continue;
                };

                debug!("Sent leaf connection request.");
            }
        }
    }

    Ok(())
}
