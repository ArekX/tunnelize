use std::sync::Arc;

use configuration::UdpEndpointConfig;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use tokio::io::Result;
use udp_services::UdpServices;

use crate::{common::channel::RequestReceiver, server::services::Services};

use super::messages::EndpointChannelRequest;

mod channel_handler;
pub mod configuration;
mod leaf_endpoint;
mod tunnel_host;
mod udp_services;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UdpEndpointInfo {
    pub assigned_hostname: String,
}

pub async fn start(
    services: Arc<Services>,
    name: String,
    config: UdpEndpointConfig,
    mut channel_rx: RequestReceiver<EndpointChannelRequest>,
) -> Result<()> {
    let udp_services = Arc::new(UdpServices::new(
        config.clone(),
        name.clone(),
        services.clone(),
    ));

    for port in config.reserve_ports_from..=config.reserve_ports_to {
        let udp_services = udp_services.clone();
        tokio::spawn(async move {
            if let Err(e) = leaf_endpoint::start(port, udp_services).await {
                error!("Failed to create leaf endpoint: {}", e);
            }
        });
    }

    let cancel_token = services.get_cancel_token();

    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                info!("Endpoint '{}' has been shutdown", name);
                return Ok(());
            }
            request = channel_rx.wait_for_requests() => {
                match request {
                    Some(request) => {
                        debug!("Received endpoint message");
                        if let Err(e) = channel_handler::handle(request, &udp_services).await {
                            error!("Failed to handle endpoint message: {}", e);
                        }
                    },
                    None => {
                        info!("Endpoint '{}' channel has been shutdown", name);
                        return Ok(());
                    }
                }
            }
        }
    }
}
