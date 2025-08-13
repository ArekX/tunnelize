use std::sync::Arc;

use log::{debug, error};
use tokio::io::Result;
use tokio_util::sync::CancellationToken;

use crate::common::transport::MessageError;
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
        error!("Failed to authenticate: {}", e);
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
            result = connection_stream.read_message::<TunnelRequestMessage>() => {
                match result {
                    Ok(message) => {
                        incoming_requests::handle(&services, &mut connection_stream, message).await;
                    },
                    Err(MessageError::ConnectionClosed) => {
                        debug!("Connection closed.");
                        cancel_token.cancel();
                    },
                    Err(e) => {
                        error!("Failed to read message from server: {}", e);
                        continue;
                    }
                }
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(30)) => {
                if let Err(e) = outgoing_requests::send_heartbeat(&services, &mut connection_stream).await {
                    error!("Failed to send heartbeat: {}", e);
                    cancel_token.cancel();
                    return Err(e);
                }
            }
        }
    }
}
