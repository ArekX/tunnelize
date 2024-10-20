use std::sync::Arc;

use crate::{common::connection::ConnectionStream, server::services::Services};

use super::configuration::TcpEndpointConfig;
use tokio::io::Result;

pub async fn handle(
    mut stream: ConnectionStream,
    name: &str,
    config: &TcpEndpointConfig,
    services: &Arc<Services>,
) -> Result<()> {
    todo!()
}
