use std::sync::Arc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{common::data_request::DataRequest, connect_data_response, tunnel::services::Services};

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
    mut request: &mut DataRequest<InitLinkRequest>,
) {
    let config = services.get_config();

    // TODO: Implement the rest of the function
}
