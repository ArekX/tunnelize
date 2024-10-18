use std::{io::ErrorKind, sync::Arc};

use log::error;
use uuid::Uuid;

use crate::{
    server::incoming_requests::{
        InitLinkRequest as ServerInitLinkRequest, InitLinkResponse as ServerInitLinkResponse,
    },
    tunnel::{client::create_server_connection, services::Services},
};

pub async fn start_link_session(
    services: Arc<Services>,
    proxy_id: Uuid,
    session_id: Uuid,
) -> tokio::io::Result<()> {
    let config = services.get_config();

    let mut forward_connection = match services
        .get_proxy_manager()
        .await
        .create_forward_connection(&proxy_id)
        .await
    {
        Ok(connection) => connection,
        Err(e) => {
            error!("Failed to create forward connection: {:?}", e);
            return Err(e);
        }
    };

    let mut server_connection = create_server_connection(&config).await?;

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
        if let Err(e) = forward_connection.pipe_to(&mut server_connection).await {
            error!("Relay session failed: {:?}", e);
        }
    });

    Ok(())
}
