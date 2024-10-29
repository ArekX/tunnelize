use std::ops::ControlFlow;
use std::sync::Arc;

use log::{debug, error, info};
use tokio::io::{self};
use tokio::{io::Result, net::TcpStream};
use tokio_util::sync::CancellationToken;

use crate::common::address::resolve_hostname;
use crate::common::connection::ConnectionStream;
use crate::common::encryption::ClientTlsEncryption;
use crate::tunnel::configuration::Encryption;
use crate::tunnel::incoming_requests;
use crate::tunnel::incoming_requests::TunnelRequestMessage;
use crate::tunnel::outgoing_requests;

use super::configuration::TunnelConfiguration;
use super::services::Services;

pub async fn create_server_connection(config: &TunnelConfiguration) -> Result<ConnectionStream> {
    let server_ip = resolve_hostname(&config.server_address)?;

    debug!("Resolved server {} -> {}", config.server_address, server_ip);

    match TcpStream::connect(server_ip.clone()).await {
        Ok(stream) => {
            match config.encryption {
                Encryption::Tls { .. } | Encryption::NativeTls => {
                    let tls =
                        ClientTlsEncryption::new(config.encryption.to_encryption_type()).await;

                    info!("Connected to (TLS) server at {}", config.server_address);

                    // TODO: needs testing and fixing
                    return Ok(tls.connect(stream, config.server_address.clone()).await?);
                }
                Encryption::None => {
                    info!("Connected to server at {}", config.server_address);
                    return Ok(ConnectionStream::from(stream));
                }
            }
        }
        Err(e) if e.kind() == io::ErrorKind::ConnectionRefused => {
            error!("Connection refused by server at {}", config.server_address);
            return Err(e);
        }
        Err(e) => {
            debug!("Error connecting to server: {:?}", e);
            return Err(e);
        }
    }
}

pub async fn start(services: Arc<Services>, cancel_token: CancellationToken) -> Result<()> {
    let config = services.get_config();

    let mut connection_stream = create_server_connection(&config).await?;

    if let Err(e) =
        outgoing_requests::authenticate_tunnel(&services, &config, &mut connection_stream).await
    {
        error!("Failed to authenticate: {:?}", e);
        return Err(e);
    }

    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {

                debug!("Ending tunnel...");
                connection_stream.shutdown().await;
                debug!("Hub server stopped.");

                return Ok(());
            }
            flow = connection_stream.wait_for_data() => {
                match flow {
                    Ok(ControlFlow::Break(_)) => {
                        println!("Server closed the connection.");
                        return Ok(());
                    }
                    Ok(ControlFlow::Continue(_)) => {}
                    Err(e) => {
                        error!("Error waiting for messages: {:?}", e);
                        return Err(e);
                    }
                }
            }
        }

        let message: TunnelRequestMessage = match connection_stream.read_message().await {
            Ok(message) => message,
            Err(e) => {
                error!("Failed to read message from server: {}", e);
                continue;
            }
        };

        incoming_requests::handle(&services, &mut connection_stream, message).await;
    }
}
