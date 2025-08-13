use std::sync::Arc;

use tokio::io::{self, Result};

use crate::common::connection::Connection;
use crate::server::incoming_requests::{HeartbeatRequest, HeartbeatResponse};

use crate::tunnel::services::Services;

pub async fn send_heartbeat(services: &Arc<Services>, server: &mut Connection) -> Result<()> {
    let Some(tunnel_id) = services.get_tunnel_data().await.get_tunnel_id() else {
        return Err(io::Error::other("Tunnel ID is not set"));
    };

    let Ok(response): std::result::Result<HeartbeatResponse, std::io::Error> =
        server.request_message(HeartbeatRequest { tunnel_id }).await
    else {
        let mut tunnel_data = services.get_tunnel_data().await;
        tunnel_data.record_failed_heartbeat();

        if tunnel_data.too_many_failed_heartbeats() {
            return Err(io::Error::other(
                "Too many failed heartbeat requests to server, server is unavailable.",
            ));
        }

        return Ok(());
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

            services.get_tunnel_data().await.record_success_heartbeat();
        }
    }

    Ok(())
}
