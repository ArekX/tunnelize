use std::{
    io::{Error, ErrorKind},
    sync::Arc,
};

use configuration::HttpEndpointConfig;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use tokio::{io::Result, net::TcpListener};
use tunnel_host::TunnelHost;

use crate::{
    common::{channel::RequestReceiver, connection::ConnectionStream},
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

    let listener = match TcpListener::bind(config.get_bind_address()).await {
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
            client = listener.accept() => {
                match client {
                    Ok((stream, stream_address)) => {
                        info!("Accepted connection from client: {}", stream_address);
                        if let Err(e) = data_handler::handle(ConnectionStream::from(stream), &tunnel_host, &name, &config, &services).await {
                            error!("Failed to handle client request: {}", e);
                        }
                    },
                    Err(e) => {
                        error!("Failed to accept client connection: {}", e);
                        continue;
                    }
                };
            }
        }
    }
}
