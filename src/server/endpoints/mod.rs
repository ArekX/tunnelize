use std::sync::Arc;

use http::HttpEndpointInfo;
use log::{debug, error};

use super::{
    configuration::EndpointConfiguration,
    services::{Endpoint, EndpointMessage, Services},
};
use tokio::{io::Result, sync::mpsc};

pub mod http;

#[derive(Debug)]
pub enum EndpointInfo {
    Http(HttpEndpointInfo),
}

macro_rules! start_endpoint {
    ($service: expr, $services: ident, $name: ident, $config: ident, $channel_rx: ident) => {{
        let services = $services.clone();
        let name = $name.clone();
        let config = $config.clone();
        tokio::spawn(async move {
            if let Err(e) = $service(services, name.clone(), config, $channel_rx).await {
                error!("Error occurred while running endpoint '{}'", name);
                debug!("Error: {:?}", e);
            }
        })
    }};
}

pub async fn start_endpoints(services: Arc<Services>) -> Result<()> {
    let config = services.get_config();

    let mut endpoint_manager = services.get_endpoint_manager().await;

    for (service_name, endpoint_config) in config.endpoints.iter() {
        // FIXME: This channel needs to be created elsewhere
        let (channel_tx, channel_rx) = mpsc::channel::<EndpointMessage>(100);

        endpoint_manager.add_endpoint(Endpoint::new(
            service_name.clone(),
            endpoint_config.clone(),
            channel_tx,
        ));

        match endpoint_config {
            EndpointConfiguration::Http(http_config) => {
                start_endpoint!(http::start, services, service_name, http_config, channel_rx);
            }
        }
    }

    Ok(())
}
