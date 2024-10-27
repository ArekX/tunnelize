use std::{sync::Arc, time::Duration};

use activity_tracker::ActivityTracker;
use configuration::UdpEndpointConfig;
use log::{debug, error, info};
use messages::UdpChannelRequest;
use serde::{Deserialize, Serialize};
use tokio::{io::Result, sync::Mutex, time::interval};
use tunnel_host::TunnelHost;

use crate::{
    common::channel::{create_channel, RequestReceiver},
    server::services::Services,
};

use super::messages::EndpointChannelRequest;

mod activity_tracker;
mod channel_handler;
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
    let activity_tracker = Arc::new(Mutex::new(ActivityTracker::new()));
    let udp_config = Arc::new(config);

    let (leaf_hub_tx, mut leaf_hub_rx) = create_channel::<UdpChannelRequest>();

    for port in udp_config.reserve_ports_from..=udp_config.reserve_ports_to {
        let hub_tx = leaf_hub_tx.clone();
        let services = services.clone();
        let config = udp_config.clone();
        let activity_tracker = activity_tracker.clone();
        tokio::spawn(async move {
            if let Err(e) =
                leaf_endpoint::start(port, hub_tx, config, activity_tracker, services).await
            {
                error!("Failed to create leaf endpoint: {}", e);
            }
        });
    }

    tokio::spawn(start_cleanup_handler(
        udp_config.clone(),
        activity_tracker.clone(),
    ));

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

async fn start_cleanup_handler(
    confg: Arc<UdpEndpointConfig>,
    activity_tracker: Arc<Mutex<ActivityTracker>>,
) {
    let mut interval = interval(Duration::from_secs(confg.inactivity_timeout));

    loop {
        interval.tick().await;

        activity_tracker
            .lock()
            .await
            .cancel_all_after_timeout(confg.inactivity_timeout)
            .await
    }
}
