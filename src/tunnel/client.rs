use std::ops::ControlFlow;
use std::sync::Arc;

use log::{debug, error};
use tokio::io::{self};
use tokio::{io::Result, net::TcpStream};
use tokio_util::sync::CancellationToken;

use crate::common::address::resolve_hostname;
use crate::common::connection::ConnectionStream;
use crate::tunnel::incoming_requests;
use crate::tunnel::incoming_requests::TunnelRequestMessage;
use crate::tunnel::outgoing_requests;

use super::configuration::TunnelConfiguration;
use super::services::Services;

pub async fn create_server_connection(config: &TunnelConfiguration) -> Result<ConnectionStream> {
    let server_ip = resolve_hostname(&config.server_host)?;

    debug!("Resolved server {} -> {}", config.server_host, server_ip);

    let server = match TcpStream::connect(server_ip.clone()).await {
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

    Ok(ConnectionStream::from(server))
}

pub async fn start(services: Arc<Services>, cancel_token: CancellationToken) -> Result<()> {
    let config = services.get_config();

    let mut connection_stream = create_server_connection(&config).await?;

    if let Err(e) =
        outgoing_requests::authenticate_tunnel(&services, &config, &mut connection_stream).await
    {
        error!("Failed to authenticate: {:?}", e);
        return Err(e);
    }

    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                debug!("Hub server stopped.");
                return Ok(());
            }
            flow = connection_stream.wait_for_data() => {
                match flow {
                    Ok(ControlFlow::Break(_)) => {
                        println!("Server closed the connection.");
                        return Ok(());
                    }
                    Ok(ControlFlow::Continue(_)) => {}
                    Err(e) => {
                        error!("Error waiting for messages: {:?}", e);
                        return Err(e);
                    }
                }
            }
        }

        let message: TunnelRequestMessage = match connection_stream.read_message().await {
            Ok(message) => message,
            Err(e) => {
                error!("Failed to read message from server: {}", e);
                continue;
            }
        };

        connection_stream =
            incoming_requests::handle(services.clone(), connection_stream, message).await;
    }
}
