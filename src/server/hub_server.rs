use std::{sync::Arc, time::Duration};

use log::{debug, error};
use tokio::time::timeout;
use tokio_util::sync::CancellationToken;

use crate::{
    common::tcp_server::TcpServer,
    server::incoming_requests::{self, ServerRequestMessage},
};
use tokio::io::Result;

use super::{endpoints, services::Services};

pub async fn start(services: Arc<Services>, cancel_token: CancellationToken) -> Result<()> {
    let config = services.get_config();

    let server = match TcpServer::new(
        config.get_server_address(),
        config.get_server_port(),
        config.get_encryption(),
    )
    .await
    {
        Ok(listener) => listener,
        Err(e) => {
            error!("Failed to bind client listener: {}", e);
            return Ok(());
        }
    };

    if let Err(e) = endpoints::start_endpoints(services.clone()).await {
        error!("Failed to start endpoints: {}", e);
        return Ok(());
    }

    let max_input_wait = u64::from(config.get_max_tunnel_input_wait());

    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                debug!("Hub server stopped.");
                return Ok(());
            }
            result = server.listen_for_connection() => {
                let (pending, address) = match result {
                    Ok(accepted) => accepted,
                    Err(e) => {
                        error!("Failed to accept client connection: {}", e);
                        continue;
                    }
                };

                let services = services.clone();

                tokio::spawn(async move {
                    let mut connection_stream = match pending.establish().await {
                        Ok(connection) => connection,
                        Err((e, _)) => {
                            error!("Failed to establish connection with client {}: {}", address, e);
                            return;
                        }
                    };

                    debug!("Accepted connection from client: {}, protocol: {}", address, connection_stream.get_protocol());

                    let message: ServerRequestMessage = match timeout(
                        Duration::from_secs(max_input_wait),
                        connection_stream.read_message(),
                    )
                    .await
                    {
                        Ok(Ok(message)) => message,
                        Ok(Err(e)) => {
                            error!("Failed to read message from client: {}", e);
                            return;
                        }
                        Err(_) => {
                            error!("Client {} did not send a request within {} seconds", address, max_input_wait);
                            connection_stream.shutdown().await;
                            return;
                        }
                    };

                    incoming_requests::handle(services, connection_stream, address, message).await;
                });
            }
        }
    }
}
