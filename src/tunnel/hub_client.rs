use std::sync::Arc;

use log::{debug, info};
use tokio::io;
use tokio::{io::Result, net::TcpStream};
use tokio_util::sync::CancellationToken;

use crate::common::address::resolve_hostname;

use super::services::Services;

pub async fn start(services: Arc<Services>, cancel_token: CancellationToken) -> Result<()> {
    let config = services.get_config();

    let server_ip = resolve_hostname(&config.server_host)?;

    let mut server = match TcpStream::connect(server_ip.clone()).await {
        Ok(stream) => stream,
        Err(e) if e.kind() == io::ErrorKind::ConnectionRefused => {
            info!(
                "Connection refused by server at {} ({})",
                config.server_host, server_ip
            );
            return Err(e);
        }
        Err(e) => {
            debug!("Error connecting to server: {:?}", e);
            return Err(e);
        }
    };

    if let Err(e) = authenticate_with_server(services.clone(), &mut server).await {
        debug!("Error authenticating with server: {:?}", e);
        return Err(e);
    }

    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                debug!("Hub server stopped.");
                return Ok(());
            }
            readable = server.readable() => {
                if let Err(e) = readable {
                    debug!("Error reading from server: {:?}", e);
                    return Err(e);
                }
            }
        }
    }
}

async fn authenticate_with_server(services: Arc<Services>, server: &mut TcpStream) -> Result<()> {
    Ok(())
}
