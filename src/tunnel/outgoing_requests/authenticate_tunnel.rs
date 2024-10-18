use std::sync::Arc;

use log::info;
use tokio::io::{self, Result};

use crate::common::connection::ConnectionStream;
use crate::server::incoming_requests::{InitTunelRequest, InitTunnelResponse, InputProxy};

use crate::tunnel::configuration::TunnelConfiguration;
use crate::tunnel::services::Services;

pub async fn authenticate_tunnel(
    services: &Arc<Services>,
    config: &Arc<TunnelConfiguration>,
    server: &mut ConnectionStream,
) -> Result<()> {
    let auth_response: InitTunnelResponse = server
        .request_message(InitTunelRequest {
            endpoint_key: config.endpoint_key.clone(),
            admin_key: config.admin_key.clone(),
            proxies: get_input_proxies(services, config).await,
        })
        .await?;

    match auth_response {
        InitTunnelResponse::Accepted {
            tunnel_id,
            endpoint_info,
        } => {
            services.get_tunnel_data().await.set_tunnel_id(tunnel_id);

            info!("Tunnel accepted: {}", tunnel_id);
            info!("Endpoints accepted: {:?}", endpoint_info);
        }
        InitTunnelResponse::Rejected { reason } => {
            return Err(io::Error::new(io::ErrorKind::Other, reason));
        }
    }

    Ok(())
}

async fn get_input_proxies(
    services: &Arc<Services>,
    config: &Arc<TunnelConfiguration>,
) -> Vec<InputProxy> {
    let mut proxy_manager = services.get_proxy_manager().await;
    let mut results = vec![];
    for proxy in config.proxies.iter() {
        let proxy_id = proxy_manager.add_proxy(&proxy);

        results.push(InputProxy {
            proxy_id,
            endpoint_name: proxy.endpoint_name.clone(),
            proxy: proxy.config.clone(),
        });
    }

    results
}
