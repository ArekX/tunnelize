use std::{sync::Arc, time::Duration};

use configuration::UdpEndpointConfig;
use log::{debug, error, info};
use messages::UdpChannelRequest;
use serde::{Deserialize, Serialize};
use tokio::{io::Result, time::interval};
use udp_services::UdpServices;

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
    let (leaf_hub_tx, mut leaf_hub_rx) = create_channel::<UdpChannelRequest>();

    let udp_services = Arc::new(UdpServices::new(
        config.clone(),
        services.clone(),
        leaf_hub_tx,
    ));

    for port in config.reserve_ports_from..=config.reserve_ports_to {
        let udp_services = udp_services.clone();
        tokio::spawn(async move {
            if let Err(e) = leaf_endpoint::start(port, udp_services).await {
                error!("Failed to create leaf endpoint: {}", e);
            }
        });
    }

    tokio::spawn(start_cleanup_handler(udp_services.clone()));

    let cancel_token = services.get_cancel_token();

    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                udp_services.get_client_host().await.cancel_all();
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
            leaf_request = leaf_hub_rx.wait_for_requests() => {
                match leaf_request {
                    Some(request) => {
                        debug!("Received leaf endpoint message");
                        if let Err(e) = udp_channel_handler::handle(request, &name, &udp_services).await {
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

async fn start_cleanup_handler(services: Arc<UdpServices>) {
    let inactivity_timeout = services.get_config().inactivity_timeout;
    let mut interval = interval(Duration::from_secs(inactivity_timeout));

    loop {
        interval.tick().await;

        services
            .get_client_host()
            .await
            .cleanup_inactive_clients(inactivity_timeout)
            .await
    }
}
