use std::{net::SocketAddr, sync::Arc};

use log::{debug, error};
use tokio::net::{TcpListener, TcpStream};
use tokio_util::sync::CancellationToken;

use crate::{
    common::transport::read_message,
    server::{messages::ServerRequestMessage, requests::handle_server_message},
};
use tokio::io::Result;

use super::services::Services;

pub async fn start(services: Arc<Services>, cancel_token: CancellationToken) -> Result<()> {
    let config = services.get_config();

    let listener = match TcpListener::bind(format!("0.0.0.0:{}", config.server_port)).await {
        Ok(listener) => listener,
        Err(e) => {
            error!("Failed to bind client listener: {}", e);
            return Ok(());
        }
    };

    loop {
        let mut stream: TcpStream;
        let address: SocketAddr;

        tokio::select! {
            _ = cancel_token.cancelled() => {
                debug!("Hub server stopped.");
                // TOOD: Close all connections
                return Ok(());
            }
            client = listener.accept() => {
                (stream, address) = match client {
                    Ok(stream_pair) => stream_pair,
                    Err(e) => {
                        error!("Failed to accept client connection: {}", e);
                        continue;
                    }
                };
            }
        }

        debug!("Accepted connection from client: {}", address);

        let message: ServerRequestMessage = match read_message(&mut stream).await {
            Ok(message) => message,
            Err(e) => {
                error!("Failed to read message from client: {}", e);
                continue;
            }
        };

        let services = services.clone();

        tokio::spawn(async move {
            handle_server_message(services, stream, message).await;
        });
    }
}
