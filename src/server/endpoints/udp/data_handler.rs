use std::sync::Arc;

use crate::{common::connection::ConnectionStream, server::services::Services};

use super::configuration::UdpEndpointConfig;
use tokio::io::Result;

pub async fn handle(
    stream: &mut ConnectionStream,
    name: &str,
    config: &UdpEndpointConfig,
    services: &Arc<Services>,
) -> Result<()> {
    todo!()
}
