use std::sync::Arc;

use log::{debug, error};

use super::{configuration::EndpointConfiguration, services::Services};
use tokio::io::Result;

pub mod http;
pub mod messages;
pub mod monitor;
pub mod tcp;
pub mod udp;

macro_rules! start_endpoint {
    ($endpoint_config: ident, $services: ident, $service_name: ident, $channel_rx: ident, {
        $(
            $name: ident => $service: expr
        ),*
    }) => {
        match $endpoint_config {
            $(
                EndpointConfiguration::$name(config) => {
                    let services = $services.clone();
                    let name = $service_name.clone();
                    let config = config.clone();
                    log::info!("Started endpoint: {}", $service_name);
                    tokio::spawn(async move {
                        if let Err(e) = $service(services, name.clone(), config, $channel_rx).await {
                            error!("Error occurred while running endpoint '{}'", name);
                            debug!("Error: {:?}", e);
                        }
                    });
                }
            )*
        }
    };
}

pub async fn start_endpoints(services: Arc<Services>) -> Result<()> {
    let config = services.get_config();

    let mut endpoint_manager = services.get_endpoint_manager().await;

    for (service_name, endpoint_config) in config.endpoints.iter() {
        let channel_rx = endpoint_manager.add_endpoint(service_name, endpoint_config);

        start_endpoint!(endpoint_config, services, service_name, channel_rx, {
            Http => http::start,
            Tcp => tcp::start,
            Udp => udp::start,
            Monitoring => monitor::start
        });
    }

    Ok(())
}
