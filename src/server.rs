use log::{error, info};
use tokio::io::Result;

use crate::{
    configuration::{ServerConfiguration, ServerType},
    http::start_http_server,
};

pub async fn start_server(server_config: ServerConfiguration) -> Result<()> {
    let mut server_futures = Vec::new();

    for server in server_config.servers {
        match server {
            ServerType::Http(config) => {
                server_futures.push(start_http_server(server_config.tunnel_server_port, config))
            }
            _ => {
                info!("Unsupported server type, skipping.");
                continue;
            }
        }
    }

    info!("Tunnelize servers initialized and running.");

    let mut has_error = false;

    for server_future in server_futures {
        if let Err(e) = server_future.await {
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
