use std::sync::Arc;

use crate::{common::channel::Request, server::endpoints::messages::EndpointChannelRequest};

use super::{
    configuration::TcpEndpointConfig,
    tunnel_host::{self, TunnelHost},
};
use tokio::io::Result;

pub async fn handle(
    mut request: Request<EndpointChannelRequest>,
    tunnel_host: &mut TunnelHost,
    config: &Arc<TcpEndpointConfig>,
) -> Result<()> {
    match &request.data {
        EndpointChannelRequest::RegisterTunnelRequest(proxy_request) => {
            todo!() // TODO: Implement RegisterProxyRequest
        }
        EndpointChannelRequest::RemoveTunnelRequest(_) => {
            todo!() // TODO: Implement RemoveTunnelRequest
        }
    }
}
