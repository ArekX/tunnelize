use log::{error, info};
use tokio::io::Result;

use crate::{
    configuration::{ServerConfiguration, ServiceType},
    http::start_http_server,
};

pub async fn start_server(server_config: ServerConfiguration) -> Result<()> {
    let mut services = Vec::new();

    for (_, server) in server_config.services {
        match server {
            ServiceType::Http(config) => services.push(tokio::spawn(async move {
                start_http_server(server_config.hub_server_port, config).await
            })),
            _ => {
                info!("Unsupported server type, skipping.");
                continue;
            }
        }
    }

    info!("Tunnelize servers initialized and running.");

    let mut has_error = false;

    for service in services {
        if let Err(e) = service.await {
            error!("Error procesing tunnel server: {}", e);
            has_error = true;
        }
    }

    if has_error {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "One or more servers failed.",
        ));
    }

    Ok(())
}
