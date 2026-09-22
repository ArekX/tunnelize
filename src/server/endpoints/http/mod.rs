use std::{io::ErrorKind, sync::Arc};

use configuration::HttpEndpointConfig;
use log::{debug, error, info};
use proxy::HttpProxy;
use serde::{Deserialize, Serialize};
use tokio::io::Result;
use tokio::sync::RwLock;
use tunnel_host::TunnelHost;

use crate::{
    common::{channel::RequestReceiver, configuration::ServerEncryption, tcp_server::TcpServer},
    server::services::Services,
};

use super::messages::EndpointChannelRequest;

mod channel_handler;
pub mod configuration;
mod link_connector;
mod proxy;
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
    let tunnel_host = Arc::new(RwLock::new(TunnelHost::new(&config)));
    let config = Arc::new(config);

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

    // Offer HTTP/2 to TLS clients so browsers multiplex all their requests
    // over one connection instead of queueing behind their 6-connection limit.
    let alpn = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    let server =
        match TcpServer::new_with_alpn(config.get_address(), config.port, encryption, alpn).await {
            Ok(listener) => listener,
            Err(e) => {
                error!("Failed to bind client listener: {}", e);
                return Ok(());
            }
        };

    let proxy = Arc::new(HttpProxy::new(
        services.clone(),
        name.clone(),
        config.clone(),
        tunnel_host.clone(),
    ));

    let cancel_token = services.get_cancel_token();

    loop {
        tokio::select! {
            request = channel_rx.wait_for_requests() => {
                match request {
                    Some(request) => {
                        debug!("Received endpoint message");
                        let mut tunnel_host = tunnel_host.write().await;
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
                let (pending, stream_address) = match client {
                    Ok(accepted) => accepted,
                    Err(e) => {
                        error!("Failed to accept client connection: {}", e);
                        continue;
                    }
                };

                debug!("Accepted connection from client: {}", stream_address);

                let proxy = proxy.clone();
                tokio::spawn(async move {
                    match pending.establish().await {
                        Ok(connection) => {
                            if let Err(e) = proxy.serve(connection, stream_address).await {
                                debug!("Client connection {} ended with error: {}", stream_address, e);
                            }
                        }
                        Err((e, Some(connection))) if e.kind() == ErrorKind::InvalidData && has_encryption => {
                            debug!("Received invalid TLS data. Probably not a TLS connection. Error: {:?}", e);
                            if let Err(e) = proxy.serve_tls_redirect(connection).await {
                                debug!("TLS redirect for {} failed: {}", stream_address, e);
                            }
                        }
                        Err((e, _)) => {
                            error!("Failed to establish client connection: {}", e);
                        }
                    }
                });
            },
            _ = cancel_token.cancelled() => {
                info!("Endpoint '{}' has been cancelled", name);
                return Ok(());
            }
        }
    }
}
