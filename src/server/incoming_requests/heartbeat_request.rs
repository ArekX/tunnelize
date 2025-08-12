use std::sync::Arc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::common::connection::Connection;

use super::super::services::Services;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HeartbeatRequest {
    pub tunnel_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum HeartbeatResponse {
    Acknowledged { tunnel_id: Uuid },
}

pub async fn process_heartbeat_request(
    services: &Arc<Services>,
    request: HeartbeatRequest,
    response_stream: &mut Connection,
) {
    services
        .get_tunnel_manager()
        .await
        .update_last_heartbeat(&request.tunnel_id);

    response_stream
        .respond_message(&HeartbeatResponse::Acknowledged {
            tunnel_id: request.tunnel_id,
        })
        .await;
}
