use std::{collections::HashMap, sync::Arc};

use crate::{
    common::channel::{OkResponse, Request},
    server::endpoints::messages::{
        EndpointChannelRequest, RegisterTunnelResponse, ResolvedEndpointInfo,
    },
    tunnel::configuration::ProxyConfiguration,
};

use super::{
    client_host::ClientHost, configuration::UdpEndpointConfig, tunnel_host::TunnelHost,
    UdpEndpointInfo,
};
use log::{debug, info};
use tokio::{io::Result, sync::Mutex};
use uuid::Uuid;

pub async fn handle(
    mut request: Request<EndpointChannelRequest>,
    tunnel_host: &mut TunnelHost,
    client_host: &Arc<Mutex<ClientHost>>,
    config: &Arc<UdpEndpointConfig>,
) -> Result<()> {
    match &mut request.data {
        EndpointChannelRequest::RegisterTunnelRequest(register_request) => {
            let tunnel_id = register_request.tunnel_id.clone();
            let mut proxy_info = HashMap::<Uuid, ResolvedEndpointInfo>::new();

            {
                let mut client_host = client_host.lock().await;
                // TODO: Check if this needed, session should somehow connect the tunnel to the client

                for session in register_request.proxy_sessions.iter() {
                    let ProxyConfiguration::Udp { desired_port } = session.config else {
                        debug!("Proxy session configuration passed is not for Udp endpoint");
                        reject_tunnel(
                            &mut request,
                            &tunnel_id,
                            tunnel_host,
                            "Invalid configuration for UDP endpoint.",
                        )
                        .await;
                        return Ok(());
                    };

                    if !tunnel_host.has_available_ports() {
                        reject_tunnel(
                            &mut request,
                            &tunnel_id,
                            tunnel_host,
                            "No available ports to be assigned.",
                        )
                        .await;
                        return Ok(());
                    }

                    let Ok(port) =
                        tunnel_host.add_tunnel(desired_port.clone(), tunnel_id, session.proxy_id)
                    else {
                        reject_tunnel(
                            &mut request,
                            &tunnel_id,
                            tunnel_host,
                            "Failed to assign port.",
                        )
                        .await;

                        return Ok(());
                    };

                    client_host.connect_tunnel(port, tunnel_id);

                    proxy_info.insert(
                        session.proxy_id,
                        ResolvedEndpointInfo::Udp(UdpEndpointInfo {
                            assigned_hostname: config.get_assigned_hostname(port),
                        }),
                    );
                }
            }

            request.respond(RegisterTunnelResponse::Accepted { proxy_info });
        }
        EndpointChannelRequest::RemoveTunnelRequest(remove_request) => {
            info!(
                "Removing tunnel ID '{}' from udp endpoint.",
                remove_request.tunnel_id
            );
            tunnel_host.remove_tunnel(&remove_request.tunnel_id);
            client_host
                .lock()
                .await
                .cleanup_by_tunnel(&remove_request.tunnel_id)
                .await;
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
