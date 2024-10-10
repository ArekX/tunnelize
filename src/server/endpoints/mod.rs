use std::{
    io::{Error, ErrorKind},
    sync::Arc,
};

use super::{configuration::EndpointConfiguration, services::Services};
use tokio::io::Result;

pub mod http;

pub async fn start_endpoints(services: Arc<Services>) -> Result<()> {
    let config = services.get_config();

    for (service_name, endpoint_config) in config.endpoints.iter() {
        match endpoint_config {
            EndpointConfiguration::Http(http_config) => {
                // TODO: Craete into endpoint manager
                if let Err(e) = http::start(services.clone(), http_config.clone()).await {
                    return Err(e);
                }
            }
        }
    }

    Ok(())
}
