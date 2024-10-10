use std::ops::ControlFlow;
use std::sync::Arc;

use log::{debug, error};
use tokio::io::{self};
use tokio::{io::Result, net::TcpStream};
use tokio_util::sync::CancellationToken;

use crate::common::address::resolve_hostname;
use crate::common::connection::ConnectionStream;
use crate::tunnel::requests;

use super::services::Services;

pub async fn start(services: Arc<Services>, cancel_token: CancellationToken) -> Result<()> {
    let config = services.get_config();

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

    let mut connection_stream = ConnectionStream::from_tcp_stream(server);

    if let Err(e) = requests::authenticate_with_server(&config, &mut connection_stream).await {
        error!("Failed to authenticate: {:?}", e);
        return Err(e);
    }

    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                debug!("Hub server stopped.");
                return Ok(());
            }
            flow = connection_stream.wait_for_messages() => {
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

        println!("Readable?");
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
