use crate::{common::channel::Request, server::endpoints::messages::EndpointChannelRequest};

use super::configuration::UdpEndpointConfig;
use tokio::io::Result;

pub async fn handle(
    mut request: Request<EndpointChannelRequest>,
    config: &UdpEndpointConfig,
) -> Result<()> {
    todo!() // TODO: Implement Tcp channel handler
}
