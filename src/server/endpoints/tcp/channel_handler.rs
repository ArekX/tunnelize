use crate::{common::channel::Request, server::endpoints::messages::EndpointChannelRequest};

use super::configuration::TcpEndpointConfig;
use tokio::io::Result;

pub async fn handle(
    mut request: Request<EndpointChannelRequest>,
    config: &TcpEndpointConfig,
) -> Result<()> {
    todo!() // TODO: Implement Tcp channel handler
}
