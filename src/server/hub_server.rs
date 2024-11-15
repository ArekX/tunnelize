use std::sync::Arc;

use log::{debug, error};
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

    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                debug!("Hub server stopped.");
                return Ok(());
            }
            result = server.listen_for_connection() => {
                match result {
                    Ok((mut connection_stream, address)) => {
                        debug!("Accepted connection from client: {}, protocol: {}", address, connection_stream.get_protocol());

                        let message: ServerRequestMessage = match connection_stream.read_message().await {
                            Ok(message) => message,
                            Err(e) => {
                                error!("Failed to read message from client: {}", e);
                                continue;
                            }
                        };

                        let services = services.clone();

                        tokio::spawn(async move {
                            incoming_requests::handle(services, connection_stream, address, message).await;
                        });
                    },
                    Err((e, _)) => {
                        error!("Failed to accept client connection: {}", e);
                        continue;
                    }
                };
            }
        }
    }
}
