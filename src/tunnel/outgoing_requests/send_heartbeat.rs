use std::sync::Arc;

use tokio::io::{self, Result};

use crate::common::connection::Connection;
use crate::server::incoming_requests::{HeartbeatRequest, HeartbeatResponse};
use crate::server::session::messages::TunnelSessionMessage;

use crate::tunnel::services::Services;

pub async fn send_heartbeat(services: &Arc<Services>, server: &mut Connection) -> Result<()> {
    let tunnel_id = {
        let mut tunnel_data = services.get_tunnel_data().await;

        let Some(tunnel_id) = tunnel_data.get_tunnel_id() else {
            return Err(io::Error::other("Tunnel ID is not set"));
        };

        if tunnel_data.is_heartbeat_pending() {
            tunnel_data.record_failed_heartbeat();

            if tunnel_data.too_many_failed_heartbeats() {
                return Err(io::Error::other(
                    "Too many failed heartbeat requests to server, server is unavailable.",
                ));
            }
        }

        tunnel_data.mark_heartbeat_sent();

        tunnel_id
    };

    if let Err(e) = server
        .write_message(&TunnelSessionMessage::HeartbeatRequest(HeartbeatRequest {
            tunnel_id,
        }))
        .await
    {
        return Err(io::Error::other(format!("Failed to send heartbeat: {e}")));
    }

    Ok(())
}

pub async fn handle_heartbeat_response(
    services: &Arc<Services>,
    response: HeartbeatResponse,
) -> Result<()> {
    let mut tunnel_data = services.get_tunnel_data().await;

    let Some(tunnel_id) = tunnel_data.get_tunnel_id() else {
        return Err(io::Error::other("Tunnel ID is not set"));
    };

    match response {
        HeartbeatResponse::Acknowledged {
            tunnel_id: response_tunnel_id,
        } => {
            if response_tunnel_id != tunnel_id {
                return Err(io::Error::other(
                    "Tunnel ID mismatch in heartbeat response. Server invalid or wrong data received.",
                ));
            }

            tunnel_data.record_success_heartbeat();
        }
    }

    Ok(())
}
