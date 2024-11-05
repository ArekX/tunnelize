use std::sync::Arc;

use configuration::TcpEndpointConfig;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use tcp_services::TcpServices;
use tokio::io::Result;

use crate::{common::channel::RequestReceiver, server::services::Services};

use super::messages::EndpointChannelRequest;

mod channel_handler;
pub mod configuration;
mod leaf_endpoint;
mod tcp_services;
mod tunnel_host;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TcpEndpointInfo {
    pub assigned_hostname: String,
}

pub async fn start(
    services: Arc<Services>,
    name: String,
    config: TcpEndpointConfig,
    mut channel_rx: RequestReceiver<EndpointChannelRequest>,
) -> Result<()> {
    let services = Arc::new(TcpServices::new(config, name, services)?);

    let config = services.get_config();

    for port in config.reserve_ports_from..=config.reserve_ports_to {
        let services = services.clone();
        tokio::spawn(async move {
            if let Err(e) = leaf_endpoint::start(port, services).await {
                error!("Failed to create leaf endpoint: {}", e);
            }
        });
    }

    let cancel_token = services.get_cancel_token();

    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                info!("Endpoint '{}' has been shutdown", services.get_endpoint_name());
                return Ok(());
            }
            request = channel_rx.wait_for_requests() => {
                match request {
                    Some(request) => {
                        debug!("Received endpoint message");
                        if let Err(e) = channel_handler::handle(request, &services).await {
                            error!("Failed to handle endpoint message: {}", e);
                        }
                    },
                    None => {
                        info!("Endpoint '{}' channel has been shutdown", services.get_endpoint_name());
                        cancel_token.cancel();
                        return Ok(());
                    }
                }
            }
        }
    }
}
