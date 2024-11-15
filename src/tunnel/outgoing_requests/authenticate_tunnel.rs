use std::sync::Arc;

use log::error;
use tokio::io::{self, Result};

use crate::common::connection::Connection;
use crate::server::endpoints::messages::ResolvedEndpointInfo;
use crate::server::incoming_requests::{InitTunelRequest, InitTunnelResponse, InputProxy};

use crate::tunnel::configuration::TunnelConfiguration;
use crate::tunnel::services::Services;

pub async fn authenticate_tunnel(
    services: &Arc<Services>,
    config: &Arc<TunnelConfiguration>,
    server: &mut Connection,
) -> Result<()> {
    let input_proxies = process_input_proxies(services, config).await;

    let auth_response: InitTunnelResponse = server
        .request_message(InitTunelRequest {
            name: config.name.clone(),
            tunnel_key: config.tunnel_key.clone(),
            admin_key: config.monitor_key.clone(),
            proxies: input_proxies.clone(),
        })
        .await?;

    match auth_response {
        InitTunnelResponse::Accepted {
            tunnel_id,
            endpoint_info,
        } => {
            services.get_tunnel_data().await.set_tunnel_id(tunnel_id);

            let proxy_manager = services.get_proxy_manager().await;

            for (proxy_id, endpoint_info) in endpoint_info.iter() {
                let Some((address, port)) = proxy_manager.get_forward_address(proxy_id) else {
                    error!("Failed to get proxy for proxy_id: {}", proxy_id);
                    return Err(io::Error::new(io::ErrorKind::Other, "Failed to get proxy"));
                };

                println!(
                    "Proxy: {}:{} -> {}",
                    address,
                    port,
                    match endpoint_info {
                        ResolvedEndpointInfo::Http(info) => {
                            info.assigned_url.clone()
                        }
                        ResolvedEndpointInfo::Tcp(info) => {
                            info.assigned_hostname.clone()
                        }
                        ResolvedEndpointInfo::Udp(info) => {
                            info.assigned_hostname.clone()
                        }
                    }
                );
            }

            // TODO: Implement tunnel --service command. Effectively, runs tunneling in the background, and exposes api for managing the tunnel.
        }
        InitTunnelResponse::Rejected { reason } => {
            return Err(io::Error::new(io::ErrorKind::Other, reason));
        }
    }

    Ok(())
}

async fn process_input_proxies(
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
            proxy: proxy.endpoint_config.clone(),
        });
    }

    results
}
