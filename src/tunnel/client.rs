use std::sync::Arc;

use bincode::Config;
use log::{debug, info};
use tokio::io::{self, AsyncReadExt};
use tokio::{io::Result, net::TcpStream};
use tokio_util::sync::CancellationToken;

use crate::common::address::resolve_hostname;
use crate::common::request::send_request;
use crate::server::messages::{ServerRequestMessage, ServerResponseMessage};

use super::configuration::TunnelConfiguration;
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

    if let Err(e) = authenticate_with_server(&config, &mut server).await {
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

        if is_closed(&mut server).await {
            println!("Server closed the connection.");
            cancel_token.cancel();
            return Ok(());
        }

        println!("Readable?");
    }
}

async fn is_closed(server: &mut TcpStream) -> bool {
    let mut buf = [0; 1];
    match server.peek(&mut buf).await {
        Ok(0) => true,
        Ok(_) => false,
        Err(e) if e.kind() == io::ErrorKind::WouldBlock => false,
        Err(_) => true,
    }
}

async fn authenticate_with_server(
    config: &Arc<TunnelConfiguration>,
    server: &mut TcpStream,
) -> Result<()> {
    let auth_response: ServerResponseMessage = send_request(
        server,
        &ServerRequestMessage::AuthTunnelRequest {
            endpoint_key: None,
            admin_key: None,
            proxies: vec![],
        },
    )
    .await?;

    println!("Response: {:?}", auth_response);

    Ok(())
}
