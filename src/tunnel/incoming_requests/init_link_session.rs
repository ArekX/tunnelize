use std::sync::Arc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    common::{connection::ConnectionStream, request::DataRequest},
    connect_data_response,
    server::incoming_requests::{AuthLinkRequest, ServerRequestMessage},
    tunnel::services::Services,
};

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

connect_data_response!(InitLinkRequest, InitLinkResponse);

pub async fn handle_init_link_session(
    services: Arc<Services>,
    mut request: &mut DataRequest<InitLinkRequest>,
) {
    let config = services.get_config();

    request
        .response_stream
        .write_message(&ServerRequestMessage::AuthLink(AuthLinkRequest {
            session_id: request.data.session_id,
            tunnel_id: request.data.tunnel_id,
        }))
        .await
        .unwrap();
}
