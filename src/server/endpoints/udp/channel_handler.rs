use std::{collections::HashMap, sync::Arc};

use crate::{
    common::channel::{OkResponse, Request},
    server::endpoints::messages::{
        EndpointChannelRequest, RegisterTunnelResponse, ResolvedEndpointInfo,
    },
    tunnel::configuration::ProxyConfiguration,
};

use super::{udp_services::UdpServices, UdpEndpointInfo};
use log::{debug, info};
use tokio::io::Result;
use uuid::Uuid;

pub async fn handle(
    mut request: Request<EndpointChannelRequest>,
    services: &Arc<UdpServices>,
) -> Result<()> {
    match &mut request.data {
        EndpointChannelRequest::RegisterTunnelRequest(register_request) => {
            let tunnel_id = register_request.tunnel_id;
            let mut proxy_info = HashMap::<Uuid, ResolvedEndpointInfo>::new();
            let config = services.get_config();
            let mut tunnel_host = services.get_tunnel_host().await;

            for session in register_request.proxy_sessions.iter() {
                let ProxyConfiguration::Udp { desired_port, .. } = session.config else {
                    debug!("Proxy session configuration passed is not for Udp endpoint");
                    reject_tunnel(
                        &mut request,
                        &tunnel_id,
                        services,
                        "Invalid configuration for UDP endpoint.",
                    )
                    .await;
                    return Ok(());
                };

                if !tunnel_host.has_available_ports() {
                    reject_tunnel(
                        &mut request,
                        &tunnel_id,
                        services,
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
                        services,
                        "Failed to assign port.",
                    )
                    .await;

                    return Ok(());
                };

                proxy_info.insert(
                    session.proxy_id,
                    ResolvedEndpointInfo::Udp(UdpEndpointInfo {
                        assigned_hostname: config.get_assigned_hostname(port),
                    }),
                );
            }

            request.respond(RegisterTunnelResponse::Accepted { proxy_info });
        }
        EndpointChannelRequest::RemoveTunnelRequest(remove_request) => {
            info!(
                "Removing tunnel ID '{}' from udp endpoint.",
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
    services: &Arc<UdpServices>,
    reason: &str,
) {
    services.get_tunnel_host().await.remove_tunnel(tunnel_id);
    request.respond(RegisterTunnelResponse::Rejected {
        reason: reason.to_string(),
    });
}
