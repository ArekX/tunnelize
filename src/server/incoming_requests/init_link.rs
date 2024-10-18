use std::sync::Arc;

use log::debug;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{common::connection::ConnectionStream, server::services::events::ServiceEvent};

use super::super::services::Services;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InitLinkRequest {
    pub tunnel_id: Uuid,
    pub session_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum InitLinkResponse {
    Accepted,
    Rejected { reason: String },
}

pub async fn process_init_link(
    services: Arc<Services>,
    request: InitLinkRequest,
    mut response_stream: ConnectionStream,
) {
    let Some(client_id) = services
        .get_link_manager()
        .await
        .resolve_tunnel_session_client(&request.session_id, &request.tunnel_id)
    else {
        response_stream
            .respond_message(&InitLinkResponse::Rejected {
                reason: "Session not found".to_string(),
            })
            .await;
        return;
    };

    start_relay(&services, client_id.clone(), response_stream).await;

    services
        .push_event(ServiceEvent::LinkDisconnected {
            client_id,
            session_id: request.session_id,
        })
        .await;
}

pub async fn start_relay(
    services: &Arc<Services>,
    client_id: Uuid,
    mut response_stream: ConnectionStream,
) {
    let Some(mut client_link) = services
        .get_client_manager()
        .await
        .take_client_link(&client_id)
    else {
        response_stream
            .respond_message(&InitLinkResponse::Rejected {
                reason: "Client not found".to_string(),
            })
            .await;
        return;
    };

    response_stream
        .respond_message(&InitLinkResponse::Accepted)
        .await;

    if let Some(data) = client_link.initial_tunnel_data {
        if let Err(e) = response_stream.write_all(&data).await {
            debug!("Error writing initial tunnel data: {:?}", e);
            return;
        }
    }

    if let Err(e) = response_stream.pipe_to(&mut client_link.stream).await {
        debug!("Error linking session: {:?}", e);
    }
}
