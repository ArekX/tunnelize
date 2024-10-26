use std::sync::Arc;

use client_host::ClientHost;
use configuration::UdpEndpointConfig;
use log::{debug, error, info};
use messages::UdpChannelRequest;
use serde::{Deserialize, Serialize};
use tokio::{io::Result, sync::Mutex};
use tunnel_host::TunnelHost;

use crate::{
    common::channel::{create_channel, RequestReceiver},
    server::services::Services,
};

use super::messages::EndpointChannelRequest;

mod channel_handler;
mod client_host;
pub mod configuration;
mod leaf_endpoint;
mod messages;
mod tunnel_host;
mod udp_channel_handler;

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
    let mut tunnel_host = TunnelHost::new(&config);
    let client_host = Arc::new(Mutex::new(ClientHost::new()));
    let udp_config = Arc::new(config);

    let (leaf_hub_tx, mut leaf_hub_rx) = create_channel::<UdpChannelRequest>();

    for port in udp_config.reserve_ports_from..=udp_config.reserve_ports_to {
        let hub_tx = leaf_hub_tx.clone();
        let services = services.clone();
        let config = udp_config.clone();
        let client_host = client_host.clone();
        tokio::spawn(async move {
            if let Err(e) = leaf_endpoint::start(port, hub_tx, config, client_host, services).await
            {
                error!("Failed to create leaf endpoint: {}", e);
            }
        });
    }

    // TODO: Add a supervisor to cleanup timed out UDP connections

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
                        if let Err(e) = channel_handler::handle(request, &mut tunnel_host, &udp_config).await {
                            error!("Failed to handle endpoint message: {}", e);
                        }
                    },
                    None => {
                        info!("Endpoint '{}' channel has been shutdown", name);
                        return Ok(());
                    }
                }
            }
            leaf_request = leaf_hub_rx.wait_for_requests() => {
                match leaf_request {
                    Some(request) => {
                        debug!("Received leaf endpoint message");
                        if let Err(e) = udp_channel_handler::handle(request, &name, &mut tunnel_host, &services).await {
                            error!("Failed to handle leaf endpoint message: {}", e);
                        }
                    },
                    None => {
                        info!("Leaf endpoint channel has been shutdown");
                        cancel_token.cancel();
                        return Ok(());
                    }
                }
            }

        }
    }
}
