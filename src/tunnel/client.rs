use std::sync::Arc;

use log::{debug, error, info};
use tokio::io::{self};
use tokio::sync::watch::error;
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

    debug!("Resolved server {} -> {}", config.server_host, server_ip);

    let mut server = match TcpStream::connect(server_ip.clone()).await {
        Ok(stream) => stream,
        Err(e) if e.kind() == io::ErrorKind::ConnectionRefused => {
            error!("Connection refused by server at {}", config.server_host);
            return Err(e);
        }
        Err(e) => {
            debug!("Error connecting to server: {:?}", e);
            return Err(e);
        }
    };

    println!("Connected to server at {}", config.server_host);

    if let Err(e) = authenticate_with_server(&config, &mut server).await {
        error!("Failed to authenticate: {:?}", e);
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
            endpoint_key: config.endpoint_key.clone(),
            admin_key: config.admin_key.clone(),
            proxies: vec![],
        },
    )
    .await?;

    match auth_response {
        ServerResponseMessage::AuthTunnelAccepted { tunnel_id } => {
            info!("Tunnel accepted: {}", tunnel_id);
        }
        ServerResponseMessage::AuthTunnelRejected { reason } => {
            return Err(io::Error::new(io::ErrorKind::Other, reason));
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Invalid message received.",
            ));
        }
    }

    Ok(())
}
