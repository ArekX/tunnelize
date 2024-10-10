use std::{net::SocketAddr, sync::Arc};

use log::{debug, error};
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;

use crate::{
    common::connection::ConnectionStream,
    server::incoming_requests::{self, ServerRequestMessage},
};
use tokio::io::Result;

use super::{endpoints, services::Services};

pub async fn start(services: Arc<Services>, cancel_token: CancellationToken) -> Result<()> {
    let config = services.get_config();

    let listener = match TcpListener::bind(format!("0.0.0.0:{}", config.server_port)).await {
        Ok(listener) => listener,
        Err(e) => {
            error!("Failed to bind client listener: {}", e);
            return Ok(());
        }
    };

    if let Err(e) = endpoints::start_endpoints(services.clone()) {
        error!("Failed to start endpoints: {}", e);
        return Ok(());
    }

    loop {
        let mut connection_stream: ConnectionStream;
        let address: SocketAddr;

        tokio::select! {
            _ = cancel_token.cancelled() => {
                debug!("Hub server stopped.");
                // TOOD: Close all connections
                return Ok(());
            }
            client = listener.accept() => {
                match client {
                    Ok((stream, stream_address)) => {
                        connection_stream = ConnectionStream::from(stream);
                        address = stream_address;
                    },
                    Err(e) => {
                        error!("Failed to accept client connection: {}", e);
                        continue;
                    }
                };
            }
        }

        debug!("Accepted connection from client: {}", address);

        let message: ServerRequestMessage = match connection_stream.read_message().await {
            Ok(message) => message,
            Err(e) => {
                error!("Failed to read message from client: {}", e);
                continue;
            }
        };

        let services = services.clone();

        tokio::spawn(async move {
            incoming_requests::handle(services, connection_stream, message).await;
        });
    }
}
