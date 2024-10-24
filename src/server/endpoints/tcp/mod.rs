use std::sync::Arc;

use configuration::TcpEndpointConfig;
use log::{debug, error, info};
use messages::TcpChannelRequest;
use tokio::{io::Result, sync::Mutex};
use tunnel_host::TunnelHost;

use crate::{
    common::channel::{create_channel, RequestReceiver},
    server::services::Services,
};

use super::messages::EndpointChannelRequest;

mod channel_handler;
pub mod configuration;
mod data_handler;
mod leaf_endpoint;
mod messages;
mod tcp_channel_handler;
mod tunnel_host;

pub async fn start(
    services: Arc<Services>,
    name: String,
    config: TcpEndpointConfig,
    mut channel_rx: RequestReceiver<EndpointChannelRequest>,
) -> Result<()> {
    let mut tunnel_host = TunnelHost::new();
    let tcp_config = Arc::new(config);

    let (leaf_hub_tx, mut leaf_hub_rx) = create_channel::<TcpChannelRequest>();

    for port in tcp_config.reserve_ports_from..=tcp_config.reserve_ports_to {
        let hub_tx = leaf_hub_tx.clone();
        let services = services.clone();
        let config = tcp_config.clone();
        tokio::spawn(async move {
            if let Err(e) =
                leaf_endpoint::create_leaf_endpoint(port, hub_tx, config, services).await
            {
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
                        if let Err(e) = channel_handler::handle(request, &mut tunnel_host, &tcp_config).await {
                            error!("Failed to handle endpoint message: {}", e);
                        }
                    },
                    None => {
                        info!("Endpoint '{}' channel has been shutdown", name);
                        cancel_token.cancel();
                        return Ok(());
                    }
                }
            }
            leaf_request = leaf_hub_rx.wait_for_requests() => {
                match leaf_request {
                    Some(request) => {
                        debug!("Received leaf endpoint message");
                        if let Err(e) = tcp_channel_handler::handle(request, &tcp_config, &mut tunnel_host, &services).await {
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
