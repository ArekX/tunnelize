use std::{io::ErrorKind, sync::Arc};

use log::error;
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use uuid::Uuid;

use crate::{
    common::connection::ConnectionStream,
    server::incoming_requests::{
        InitLinkRequest as ServerInitLinkRequest, InitLinkResponse as ServerInitLinkResponse,
    },
    tunnel::{client::create_server_connection, services::Services},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InitLinkRequest {
    pub tunnel_id: Uuid,
    pub proxy_id: Uuid,
    pub session_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum InitLinkResponse {
    Accepted,
    Rejected { reason: String },
}

pub async fn process_init_link(
    services: &Arc<Services>,
    request: InitLinkRequest,
    response_stream: &mut ConnectionStream,
) {
    println!("process_init_link {}", request.proxy_id);

    let Some(address) = services
        .get_proxy_manager()
        .await
        .get_forward_address(&request.proxy_id)
    else {
        response_stream
            .respond_message(&InitLinkResponse::Rejected {
                reason: "Requested proxy not found".to_string(),
            })
            .await;
        return;
    };

    {
        if let Err(e) = start_relay(services.clone(), request.session_id, address.clone()).await {
            error!("Failed to start relay: {:?}", e);

            let message = if let ErrorKind::ConnectionRefused = e.kind() {
                format!(
                    "Connection refused, could not connect to source at {}",
                    address
                )
            } else {
                format!("Failed to start relay: {:?}", e.kind())
            };

            response_stream
                .respond_message(&InitLinkResponse::Rejected { reason: message })
                .await;
        }
    };
}

pub async fn start_relay(
    services: Arc<Services>,
    session_id: Uuid,
    address: String,
) -> tokio::io::Result<()> {
    let config = services.get_config();

    let mut forward_connection = match TcpStream::connect(address).await {
        Ok(stream) => ConnectionStream::from(stream),
        Err(e) => {
            error!("Failed to connect to forward address: {}", e);
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
