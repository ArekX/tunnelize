use std::sync::Arc;

use crate::{
    common::channel::Request,
    server::endpoints::messages::{EndpointChannelRequest, RegisterTunnelResponse},
    tunnel::configuration::ProxyConfiguration,
};

use super::{configuration::TcpEndpointConfig, tunnel_host::TunnelHost};
use log::debug;
use tokio::io::Result;

pub async fn handle(
    mut request: Request<EndpointChannelRequest>,
    tunnel_host: &mut TunnelHost,
    config: &Arc<TcpEndpointConfig>,
) -> Result<()> {
    match &mut request.data {
        EndpointChannelRequest::RegisterTunnelRequest(proxy_request) => {
            let tunnel_id = proxy_request.tunnel_id.clone();
            for session in proxy_request.proxy_sessions.iter() {
                let ProxyConfiguration::Tcp { desired_port } = session.config else {
                    debug!("Proxy session configuration passed is not for Tcp endpoint");
                    continue;
                };

                if !tunnel_host.has_available_ports() {
                    request
                        .respond(RegisterTunnelResponse::Rejected {
                            reason: "No available ports".to_string(),
                        })
                        .await;
                    tunnel_host.remove_tunnel(&tunnel_id);
                    return Ok(());
                }

                tunnel_host.add_tunnel(desired_port.clone(), tunnel_id, session.proxy_id);
            }
        }
        EndpointChannelRequest::RemoveTunnelRequest(_) => {
            todo!() // TODO: Implement RemoveTunnelRequest
        }
    }

    Ok(())
}
