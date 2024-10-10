mod configuration;

use std::sync::Arc;

use bincode::config;
pub use configuration::HttpEndpointConfig;
use tokio::io::Result;

use crate::server::{configuration::EndpointConfiguration, services::Services};

pub async fn start(services: Arc<Services>, config: HttpEndpointConfig) -> Result<()> {
    let config = services.get_config();

    Ok(())
}
