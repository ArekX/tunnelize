use std::{sync::Arc, time::Duration};

use client_host::ClientHost;
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

    tokio::spawn(start_cleanup_handler(
        udp_config.clone(),
        client_host.clone(),
    ));

    let cancel_token = services.get_cancel_token();

    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                client_host.lock().await.cancel_all();
                info!("Endpoint '{}' has been shutdown", name);
                return Ok(());
            }
            request = channel_rx.wait_for_requests() => {
                match request {
                    Some(request) => {
                        debug!("Received endpoint message");
                        if let Err(e) = channel_handler::handle(request, &mut tunnel_host, &client_host, &udp_config).await {
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

async fn start_cleanup_handler(confg: Arc<UdpEndpointConfig>, client_host: Arc<Mutex<ClientHost>>) {
    let mut interval = interval(Duration::from_secs(confg.inactivity_timeout));

    loop {
        interval.tick().await;

        client_host
            .lock()
            .await
            .cleanup_inactive_clients(confg.inactivity_timeout)
            .await
    }
}
