use log::info;
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

    for server_future in futures {
        server_future.await.unwrap();
    }

    Ok(())
}
