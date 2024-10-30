use std::ops::ControlFlow;
use std::sync::Arc;

use log::{debug, error};
use tokio::io::Result;
use tokio_util::sync::CancellationToken;

use crate::tunnel::incoming_requests;
use crate::tunnel::incoming_requests::TunnelRequestMessage;
use crate::tunnel::outgoing_requests;

use super::services::Services;

pub async fn start(services: Arc<Services>, cancel_token: CancellationToken) -> Result<()> {
    let config = services.get_config();

    let mut connection_stream = config.create_tcp_client().await?;

    if let Err(e) =
        outgoing_requests::authenticate_tunnel(&services, &config, &mut connection_stream).await
    {
        error!("Failed to authenticate: {:?}", e);
        return Err(e);
    }

    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {

                debug!("Ending tunnel...");
                connection_stream.shutdown().await;
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

        incoming_requests::handle(&services, &mut connection_stream, message).await;
    }
}
