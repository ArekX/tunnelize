use std::sync::Arc;

use log::error;
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use uuid::Uuid;

use crate::{
    common::{connection::ConnectionStream, data_request::DataRequest},
    connect_data_response,
    server::incoming_requests::InitLinkRequest as ServerInitLinkRequest,
    server::incoming_requests::InitLinkResponse as ServerInitLinkResponse,
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

connect_data_response!(InitLinkRequest -> InitLinkResponse);

pub async fn process_init_link(
    services: Arc<Services>,
    request: &mut DataRequest<InitLinkRequest>,
) {
    let Some(address) = services
        .get_proxy_manager()
        .await
        .get_forward_address(&request.data.proxy_id)
    else {
        request
            .response_stream
            .respond_message(&InitLinkResponse::Rejected {
                reason: "Requested proxy not found".to_string(),
            })
            .await;
        return;
    };

    {
        let services = services.clone();
        let session_id = request.data.session_id;
        tokio::spawn(async move {
            if let Err(e) = start_relay(services, session_id, address).await {
                error!("Failed to start relay: {:?}", e);
            }
        });
    }

    request
        .response_stream
        .respond_message(&InitLinkResponse::Accepted)
        .await;
}

pub async fn start_relay(
    services: Arc<Services>,
    session_id: Uuid,
    address: String,
) -> tokio::io::Result<()> {
    let config = services.get_config();

    let mut forward_connection = ConnectionStream::from(TcpStream::connect(address).await?);
    let mut server_connection = create_server_connection(&config).await?;

    let Some(tunnel_id) = services.get_tunnel_data().await.get_tunnel_id() else {
        error!("Tunnel ID not found.");
        return Ok(());
    };

    let Ok(response): tokio::io::Result<ServerInitLinkResponse> = server_connection
        .request_message(&ServerInitLinkRequest {
            tunnel_id,
            session_id,
        })
        .await
    else {
        error!("Failed to initiate link with tunnel server.");
        return Ok(());
    };

    if let ServerInitLinkResponse::Rejected { reason } = response {
        error!("Tunnel server link rejected: {}", reason);
        return Ok(());
    }

    if let Err(e) = forward_connection
        .link_session_with(&mut server_connection)
        .await
    {
        error!("Failed to link relay: {:?}", e);
        return Err(e);
    }

    Ok(())
}
