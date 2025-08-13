use std::{collections::HashMap, sync::Arc};

use crate::{
    common::channel::{OkResponse, Request},
    server::endpoints::messages::{
        EndpointChannelRequest, RegisterTunnelResponse, ResolvedEndpointInfo,
    },
    tunnel::configuration::ProxyConfiguration,
};

use super::{tcp_services::TcpServices, tunnel_host::TunnelHost, TcpEndpointInfo};
use log::{debug, info};
use tokio::io::Result;
use uuid::Uuid;

pub async fn handle(
    mut request: Request<EndpointChannelRequest>,
    services: &Arc<TcpServices>,
) -> Result<()> {
    match &mut request.data {
        EndpointChannelRequest::RegisterTunnelRequest(register_request) => {
            let tunnel_id = register_request.tunnel_id;
            let mut proxy_info = HashMap::<Uuid, ResolvedEndpointInfo>::new();
            let mut tunnel_host = services.get_tunnel_host().await;
            let config = services.get_config();

            for session in register_request.proxy_sessions.iter() {
                let ProxyConfiguration::Tcp { desired_port } = session.config else {
                    debug!("Proxy session configuration passed is not for Tcp endpoint");
                    reject_tunnel(
                        &mut request,
                        &tunnel_id,
                        &mut tunnel_host,
                        "Invalid configuration for TCP endpoint.",
                    )
                    .await;
                    return Ok(());
                };

                if !tunnel_host.has_available_ports() {
                    reject_tunnel(
                        &mut request,
                        &tunnel_id,
                        &mut tunnel_host,
                        "No available ports to be assigned.",
                    )
                    .await;
                    return Ok(());
                }

                let Ok(port) =
                    tunnel_host.add_tunnel(desired_port, tunnel_id, session.proxy_id)
                else {
                    reject_tunnel(
                        &mut request,
                        &tunnel_id,
                        &mut tunnel_host,
                        "Failed to assign port.",
                    )
                    .await;

                    return Ok(());
                };

                proxy_info.insert(
                    session.proxy_id,
                    ResolvedEndpointInfo::Tcp(TcpEndpointInfo {
                        assigned_hostname: config.get_assigned_hostname(port),
                    }),
                );
            }

            request.respond(RegisterTunnelResponse::Accepted { proxy_info });
        }
        EndpointChannelRequest::RemoveTunnelRequest(remove_request) => {
            info!(
                "Removing tunnel ID '{}' from tcp endpoint.",
                remove_request.tunnel_id
            );
            services
                .get_tunnel_host()
                .await
                .remove_tunnel(&remove_request.tunnel_id);
            request.respond(OkResponse);
        }
    }

    Ok(())
}

async fn reject_tunnel(
    request: &mut Request<EndpointChannelRequest>,
    tunnel_id: &Uuid,
    tunnel_host: &mut TunnelHost,
    reason: &str,
) {
    tunnel_host.remove_tunnel(tunnel_id);
    request.respond(RegisterTunnelResponse::Rejected {
        reason: reason.to_string(),
    });
}
