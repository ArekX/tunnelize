use std::{
    io::{Error, ErrorKind},
    sync::Arc,
};

use configuration::HttpEndpointConfig;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use tokio::io::Result;
use tunnel_host::TunnelHost;

use crate::{
    common::{
        channel::RequestReceiver,
        tcp_server::{ServerEncryption, TcpServer},
    },
    server::{
        configuration::{EndpointServerEncryption, ServerConfiguration},
        services::Services,
    },
};

use super::messages::EndpointChannelRequest;

mod channel_handler;
pub mod configuration;
mod data_handler;
mod protocol;
mod tunnel_host;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HttpEndpointInfo {
    pub assigned_url: String,
}

fn get_server_encryption(
    name: &str,
    config: &Arc<ServerConfiguration>,
    encryption: &EndpointServerEncryption,
) -> Result<ServerEncryption> {
    match encryption {
        EndpointServerEncryption::None => Ok(ServerEncryption::None),
        EndpointServerEncryption::CustomTls {
            cert_path,
            key_path,
        } => Ok(ServerEncryption::Tls {
            cert_path: cert_path.clone(),
            key_path: key_path.clone(),
        }),
        EndpointServerEncryption::ServerTls => {
            let (cert_path, key_path) = match config.encryption {
                ServerEncryption::Tls {
                    ref cert_path,
                    ref key_path,
                } => (cert_path, key_path),
                ServerEncryption::None => {
                    return Err(Error::new(
                        ErrorKind::InvalidInput,
                        format!("Tunnel server TLS encryption is not set, but required by monitor '{}' endpoint", name),
                    ));
                }
            };

            Ok(ServerEncryption::Tls {
                cert_path: cert_path.clone(),
                key_path: key_path.clone(),
            })
        }
    }
}

pub async fn start(
    services: Arc<Services>,
    name: String,
    config: HttpEndpointConfig,
    mut channel_rx: RequestReceiver<EndpointChannelRequest>,
) -> Result<()> {
    let mut tunnel_host = TunnelHost::new(&config);

    let encryption = match get_server_encryption(&name, &services.get_config(), &config.encryption)
    {
        Ok(encryption) => encryption,
        Err(e) => {
            error!("Failed to get server encryption: {}", e);
            return Err(e);
        }
    };

    let server = match TcpServer::new(config.get_address(), config.port, encryption).await {
        Ok(listener) => listener,
        Err(e) => {
            error!("Failed to bind client listener: {}", e);
            return Ok(());
        }
    };

    let cancel_token = services.get_cancel_token();

    loop {
        tokio::select! {
            request = channel_rx.wait_for_requests() => {
                match request {
                    Some(request) => {
                        debug!("Received endpoint message");
                        if let Err(e) = channel_handler::handle(request, &config,  &mut tunnel_host).await {
                            error!("Failed to handle endpoint message: {}", e);
                        }
                    },
                    None => {
                        info!("Endpoint '{}' channel has been shutdown", name);
                        return Ok(());
                    }
                }
            }
            client = server.listen_for_connection() => {
                match client {
                    Ok((connection, stream_address)) => {
                        info!("Accepted connection from client: {}", stream_address);
                        if let Err(e) = data_handler::handle(connection, &tunnel_host, &name, &config, &services).await {
                            error!("Failed to handle client request: {}", e);
                        }
                    },
                    Err((e, mut connection_returned)) if e.kind() == ErrorKind::InvalidData => {
                        error!("Received invalid TLS data. Probably not a TLS connection. Error: {:?}", e);

                        if let Some(mut connection) = connection_returned.take() {
                            // TODO: Read header and redirect to HTTPS version.
                            connection.close_with_data(protocol::HttpResponseBuilder::from_redirect("https://opop-test.localhost:3457/").build().as_bytes()).await;
                            continue;
                        }
                    },
                    Err(e) => {
                        error!("Failed to accept client connection: {:?}", e);
                        continue;
                    }
                };
            },
            _ = cancel_token.cancelled() => {
                info!("Endpoint '{}' has been cancelled", name);
                return Ok(());
            }
        }
    }
}
