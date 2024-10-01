use log::{debug, info};
use tokio::io::Result;

use crate::{
    configuration::{TunnelConfiguration, TunnelType},
    http::start_http_tunnel,
};

pub async fn start_server(config: TunnelConfiguration) -> Result<()> {
    let mut services = Vec::new();

    for tunnel_definition in config.tunnels {
        match tunnel_definition.tunnel {
            TunnelType::Http(tunnel_config) => {
                let tunnel_server_address = config.hub_server_address.clone();
                services.push(tokio::spawn(async move {
                    start_http_tunnel(tunnel_server_address, tunnel_config).await
                }))
            }
        }
    }

    info!("Tunnelize client initialized and running.");

    let mut has_error = false;

    for server_future in services {
        if let Err(e) = server_future.await {
            debug!("Error starting tunnel client: {}", e);
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
