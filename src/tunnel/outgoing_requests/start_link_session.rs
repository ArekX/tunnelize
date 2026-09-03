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

    let Some(proxy) = services.get_proxy_manager().await.get_proxy(&proxy_id).cloned() else {
        error!("Failed to find proxy session with id: {}", proxy_id);
        return Err(tokio::io::Error::new(
            ErrorKind::NotFound,
            "Failed to find proxy session",
        ));
    };

    let Some(tunnel_id) = services.get_tunnel_data().await.get_tunnel_id() else {
        error!("Tunnel ID not found.");
        return Err(tokio::io::Error::other(
            "Tunnel ID not found or incorrectly assigned.",
        ));
    };

    let (forward_result, server_result) = tokio::join!(
        timeout(
            Duration::from_secs(config.get_forward_connection_timeout_seconds()),
            proxy.create_forward_connection(),
        ),
        config.create_tcp_client(),
    );

    let mut server_connection = server_result?;

    let mut forward_connection = match forward_result {
        Ok(Ok(connection)) => connection,
        Ok(Err(e)) => {
            error!("Failed to create forward connection: {}", e);
            server_connection.shutdown().await;
            return Err(e);
        }
        Err(_) => {
            error!("Forward connection creation timed out.");
            server_connection.shutdown().await;
            return Err(tokio::io::Error::new(
                ErrorKind::TimedOut,
                "Forward connection creation timed out.",
            ));
        }
    };

    let auth_response: ServerInitLinkResponse = server_connection
        .request_message(ServerInitLinkRequest {
            tunnel_id,
            session_id,
        })
        .await?;

    if let ServerInitLinkResponse::Rejected { reason } = auth_response {
        error!("Tunnel server link rejected: {}", reason);
        forward_connection.shutdown().await;
        return Err(tokio::io::Error::other(reason));
    }

    if let Err(e) = forward_connection.bridge_to(&mut server_connection).await {
        error!("Relay session failed: {}", e);
    }

    forward_connection.shutdown().await;
    server_connection.shutdown().await;

    Ok(())
}
