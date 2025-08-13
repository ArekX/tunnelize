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
            EndpointChannelRequest, RegisterTunnelResponse, RemoveTunnelRequest,
            ResolvedEndpointInfo,
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
        EndpointChannelRequest::RegisterTunnelRequest(tunnel_request) => {
            let mut proxy_info = HashMap::<Uuid, ResolvedEndpointInfo>::new();

            for proxy_session in tunnel_request.proxy_sessions.iter() {
                let ProxyConfiguration::Http { desired_name } = &proxy_session.config else {
                    debug!("Proxy session configuration passed is not for Http endpoint");
                    continue;
                };
                if let Some(desired_name) = desired_name {
                    if !config.get_allow_custom_hostnames() {
                        request.respond(RegisterTunnelResponse::Rejected {
                            reason: "Custom hostnames are not allowed for this endpoint".to_owned(),
                        });
                        return Ok(());
                    }

                    if desired_name.is_empty() {
                        request.respond(RegisterTunnelResponse::Rejected {
                            reason: "Desired hostname cannot be empty".to_owned(),
                        });
                        return Ok(());
                    }

                    if desired_name.len() > 20 {
                        request.respond(RegisterTunnelResponse::Rejected {
                            reason: "Desired hostname cannot be longer than 20 characters"
                                .to_owned(),
                        });
                        return Ok(());
                    }
                }

                let hostname = tunnel_host.register_host(
                    desired_name,
                    &tunnel_request.tunnel_id,
                    &proxy_session.proxy_id,
                );

                info!(
                    "Tunnel ID '{}' connected to http endpoint with hostname '{}'",
                    tunnel_request.tunnel_id, hostname
                );

                proxy_info.insert(
                    proxy_session.proxy_id,
                    ResolvedEndpointInfo::Http(HttpEndpointInfo {
                        assigned_url: config.get_full_url(&hostname),
                    }),
                );
            }

            request.respond(RegisterTunnelResponse::Accepted { proxy_info });
        }
        EndpointChannelRequest::RemoveTunnelRequest(RemoveTunnelRequest { tunnel_id }) => {
            info!("Removing tunnel ID '{}' from http endpoint.", tunnel_id);
            tunnel_host.remove_tunnel_by_id(tunnel_id);
            request.respond(OkResponse);
        }
    }

    Ok(())
}
