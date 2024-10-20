use std::collections::HashMap;

use super::{tunnel_host::TunnelHost, HttpEndpointConfig};
use log::{debug, info};
use tokio::io::Result;
use uuid::Uuid;

use crate::{
    common::channel::{OkResponse, Request},
    server::endpoints::{
        http::HttpEndpointInfo,
        messages::{
            EndpointChannelRequest, EndpointInfo, RegisterProxyResponse, RemoveTunnelRequest,
        },
    },
    tunnel::configuration::ProxyConfiguration,
};

pub async fn handle(
    mut request: Request<EndpointChannelRequest>,
    config: &HttpEndpointConfig,
    tunnel_host: &mut TunnelHost,
) -> Result<()> {
    match &request.data {
        EndpointChannelRequest::RegisterProxyRequest(proxy_request) => {
            let mut proxy_info = HashMap::<Uuid, EndpointInfo>::new();

            for proxy_session in proxy_request.proxy_sessions.iter() {
                let ProxyConfiguration::Http { desired_name } = &proxy_session.config else {
                    debug!("Proxy session configuration passed is not for Http endpoint");
                    continue;
                };

                let hostname = tunnel_host.register_host(
                    &desired_name,
                    &proxy_request.tunnel_id,
                    &proxy_session.proxy_id,
                );

                info!(
                    "Tunnel ID '{}' connected to http endpoint with hostname '{}'",
                    proxy_request.tunnel_id, hostname
                );

                proxy_info.insert(
                    proxy_session.proxy_id,
                    EndpointInfo::Http(HttpEndpointInfo {
                        assigned_url: config.get_full_url(&hostname),
                    }),
                );
            }

            request.respond(RegisterProxyResponse { proxy_info }).await;
        }
        EndpointChannelRequest::RemoveTunnelRequest(RemoveTunnelRequest { tunnel_id }) => {
            info!("Removing tunnel ID '{}' from http endpoint.", tunnel_id);
            tunnel_host.remove_tunnel_by_id(&tunnel_id);
            request.respond(OkResponse).await;
        }
    }

    Ok(())
}
