use log::{error, info};
use tokio::io::Result;

use crate::{
    configuration::{TunnelConfiguration, TunnelType},
    http::start_http_tunnel,
};

pub async fn start_server(config: TunnelConfiguration) -> Result<()> {
    let mut futures = Vec::new();

    for server in config.tunnels {
        match server {
            TunnelType::Http(tunnel_config) => futures.push(start_http_tunnel(
                config.tunnel_server_address.clone(),
                tunnel_config,
            )),
        }
    }

    info!("Tunnelize client initialized and running.");

    let mut has_error = false;

    for server_future in futures {
        if let Err(e) = server_future.await {
            error!("Error starting tunnel client: {}", e);
            has_error = true;
        }
    }

    if has_error {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "One or more tunnel clients failed.",
        ));
    }

    Ok(())
}
