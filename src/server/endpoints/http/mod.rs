use std::{io::ErrorKind, sync::Arc};

use configuration::HttpEndpointConfig;
use log::{debug, error, info};
use protocol::{HttpRequestReader, HttpResponseBuilder};
use serde::{Deserialize, Serialize};
use tokio::io::Result;
use tunnel_host::TunnelHost;

use crate::{
    common::{
        channel::RequestReceiver, configuration::ServerEncryption, connection::Connection,
        tcp_server::TcpServer,
    },
    server::services::Services,
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

pub async fn start(
    services: Arc<Services>,
    name: String,
    config: HttpEndpointConfig,
    mut channel_rx: RequestReceiver<EndpointChannelRequest>,
) -> Result<()> {
    let mut tunnel_host = TunnelHost::new(&config);

    let encryption = match config
        .get_encryption()
        .to_encryption(&services.get_config())
    {
        Ok(encryption) => encryption,
        Err(e) => {
            error!("Failed to get server encryption: {}", e);
            return Err(e);
        }
    };

    let has_encryption = match &encryption {
        ServerEncryption::None => false,
        _ => true,
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
                    Err((e, mut connection_returned)) if e.kind() == ErrorKind::InvalidData && has_encryption => {
                        debug!("Received invalid TLS data. Probably not a TLS connection. Error: {:?}", e);

                        if let Some(mut connection) = connection_returned.take() {
                            process_tls_redirection(&mut connection, &config).await;
                            continue;
                        }
                    },
                    Err((e, _)) => {
                        error!("Failed to accept client connection: {}", e);
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

async fn process_tls_redirection(connection: &mut Connection, config: &HttpEndpointConfig) {
    let request =
        match HttpRequestReader::new(connection, config.get_max_client_input_wait_secs()).await {
            Ok(request) => request,
            Err(e) => {
                debug!(
                    "Failed to read request data within allowed time frame: {}",
                    e
                );
                return;
            }
        };

    match request.find_hostname() {
        Some(hostname) => {
            connection
                .close_with_data(
                    &HttpResponseBuilder::as_redirect(&format!(
                        "https://{}:{}",
                        hostname, config.port
                    ))
                    .build_bytes(),
                )
                .await;
        }
        None => {
            connection
                .close_with_data(&HttpResponseBuilder::as_missing_header().build_bytes())
                .await;
        }
    }
}
