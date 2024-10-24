use std::sync::Arc;

use crate::{common::channel::Request, server::services::Services};

use super::{
    configuration::TcpEndpointConfig, messages::TcpChannelRequest, tunnel_host::TunnelHost,
};
use tokio::io::Result;

pub async fn handle(
    mut request: Request<TcpChannelRequest>,
    config: &Arc<TcpEndpointConfig>,
    tunnel_host: &mut TunnelHost,
    services: &Arc<Services>,
) -> Result<()> {
    todo!() // TODO: Implement Tcp channel handler
}
