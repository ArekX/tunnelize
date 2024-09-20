use log::info;
use tokio::io::Result;

use crate::{
    configuration::{ServerConfiguration, ServerType},
    http::start_http_server,
};

pub async fn start_server(config: ServerConfiguration) -> Result<()> {
    let mut server_futures = Vec::new();

    for server in config.servers {
        match server {
            ServerType::Http(config) => server_futures.push(start_http_server(config)),
            _ => {
                info!("Unsupported server type, skipping.");
                continue;
            }
        }
    }

    info!("Tunnelize servers initialized and running.");

    for server_future in server_futures {
        server_future.await.unwrap();
    }

    Ok(())
}
