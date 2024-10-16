use std::sync::Arc;

use log::{debug, error};

use super::{configuration::EndpointConfiguration, services::Services};
use tokio::io::Result;

pub mod http;
pub mod messages;

macro_rules! start_endpoint {
    ($service: expr, $services: ident, $name: ident, $config: ident, $channel_rx: ident) => {{
        let services = $services.clone();
        let name = $name.clone();
        let config = $config.clone();
        log::info!("Started endpoint: {}", $name);
        tokio::spawn(async move {
            if let Err(e) = $service(services, name.clone(), config, $channel_rx).await {
                error!("Error occurred while running endpoint '{}'", name);
                debug!("Error: {:?}", e);
            }
        });
    }};
}

pub async fn start_endpoints(services: Arc<Services>) -> Result<()> {
    let config = services.get_config();

    let mut endpoint_manager = services.get_endpoint_manager().await;

    for (service_name, endpoint_config) in config.endpoints.iter() {
        let channel_rx = endpoint_manager.add_endpoint(service_name, endpoint_config);

        match endpoint_config {
            EndpointConfiguration::Http(http_config) => {
                start_endpoint!(http::start, services, service_name, http_config, channel_rx);
            }
        }
    }

    Ok(())
}
