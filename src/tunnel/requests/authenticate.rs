use std::sync::Arc;

use log::info;
use tokio::io::{self, Result};

use crate::common::connection::ConnectionStream;
use crate::server::messages::{ServerRequestMessage, ServerResponseMessage};
use crate::server::requests::AuthTunelRequest;
use crate::tunnel::configuration::TunnelConfiguration;

pub async fn authenticate_with_server(
    config: &Arc<TunnelConfiguration>,
    server: &mut ConnectionStream,
) -> Result<()> {
    let auth_response: ServerResponseMessage = server
        .request(&ServerRequestMessage::AuthTunnel(AuthTunelRequest {
            endpoint_key: config.endpoint_key.clone(),
            admin_key: config.admin_key.clone(),
            proxies: vec![],
        }))
        .await?;

    match auth_response {
        ServerResponseMessage::AuthTunnelAccepted { tunnel_id } => {
            info!("Tunnel accepted: {}", tunnel_id);
        }
        ServerResponseMessage::AuthTunnelRejected { reason } => {
            return Err(io::Error::new(io::ErrorKind::Other, reason));
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Invalid message received.",
            ));
        }
    }

    Ok(())
}
