use std::sync::Arc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    common::connection::Connection,
    server::{services::events::ServiceEvent, session},
};

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

pub async fn process(
    services: Arc<Services>,
    request: InitLinkRequest,
    mut response_stream: Connection,
) {
    let Some((client_id, cancel_token)) = services
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

    session::link::start(&services, client_id.clone(), response_stream, cancel_token).await;

    println!("Link session Disconnecting");

    services
        .push_event(ServiceEvent::LinkDisconnected {
            client_id,
            session_id: request.session_id,
        })
        .await;

    println!("Sent LinkDisconnected event");
}
