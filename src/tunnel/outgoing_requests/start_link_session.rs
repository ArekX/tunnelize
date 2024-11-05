use std::{io::ErrorKind, sync::Arc, time::Duration};

use log::{error, info};
use tokio::time::timeout;
use uuid::Uuid;

use crate::{
    common::data_bridge::DataBridge,
    server::incoming_requests::{
        InitLinkRequest as ServerInitLinkRequest, InitLinkResponse as ServerInitLinkResponse,
    },
    tunnel::services::Services,
};

pub async fn start_link_session(
    services: Arc<Services>,
    proxy_id: Uuid,
    session_id: Uuid,
) -> tokio::io::Result<()> {
    let config = services.get_config();

    info!("Starting link session.");
    let (mut forward_connection, context) = match timeout(
        Duration::from_secs(config.forward_connection_timeout_seconds),
        services
            .get_proxy_manager()
            .await
            .create_forward_connection(&proxy_id),
    )
    .await
    {
        Ok(Ok(connection)) => connection,
        Ok(Err(e)) => {
            error!("Failed to create forward connection: {:?}", e);
            return Err(e);
        }
        Err(_) => {
            error!("Forward connection creation timed out.");
            return Err(tokio::io::Error::new(
                ErrorKind::TimedOut,
                "Forward connection creation timed out.",
            ));
        }
    };

    let mut server_connection = config.create_tcp_client().await?;

    let Some(tunnel_id) = services.get_tunnel_data().await.get_tunnel_id() else {
        error!("Tunnel ID not found.");
        return Err(tokio::io::Error::new(
            ErrorKind::Other,
            "Tunnel ID not found or incorrectly assigned.",
        ));
    };

    let auth_response: ServerInitLinkResponse = server_connection
        .request_message(ServerInitLinkRequest {
            tunnel_id,
            session_id,
        })
        .await?;

    if let ServerInitLinkResponse::Rejected { reason } = auth_response {
        error!("Tunnel server link rejected: {}", reason);
        return Err(tokio::io::Error::new(ErrorKind::Other, reason));
    }

    tokio::spawn(async move {
        info!("Starting relay session.");
        if let Err(e) = forward_connection
            .bridge_to(&mut server_connection, context)
            .await
        {
            error!("Relay session failed: {:?}", e);
        }
    });

    Ok(())
}
