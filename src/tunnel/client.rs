use std::sync::Arc;

use log::{debug, error};
use tokio::io::Result;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::common::transport::MessageError;
use crate::server::session::messages::TunnelSessionMessage;
use crate::tunnel::incoming_requests::{self, TunnelRequestMessage};
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

    let (control_tx, mut control_rx) = mpsc::channel::<TunnelSessionMessage>(100);

    let mut heartbeat_interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
    heartbeat_interval.tick().await;

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
                    Ok(TunnelRequestMessage::InitLinkRequest(request)) => {
                        let services = services.clone();
                        let control_tx = control_tx.clone();
                        tokio::spawn(async move {
                            incoming_requests::process_init_link(services, request, control_tx).await;
                        });
                    },
                    Ok(TunnelRequestMessage::HeartbeatResponse(response)) => {
                        if let Err(e) = outgoing_requests::handle_heartbeat_response(&services, response).await {
                            error!("Invalid heartbeat response: {}", e);
                            cancel_token.cancel();
                            return Err(e);
                        }
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
            Some(message) = control_rx.recv() => {
                if let Err(e) = connection_stream.write_message(&message).await {
                    error!("Failed to send message to server: {}", e);
                }
            }
            _ = heartbeat_interval.tick() => {
                if let Err(e) = outgoing_requests::send_heartbeat(&services, &mut connection_stream).await {
                    error!("Failed to send heartbeat: {}", e);
                    cancel_token.cancel();
                    return Err(e);
                }
            }
        }
    }
}
