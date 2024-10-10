use std::{
    io::{Error, ErrorKind},
    sync::Arc,
};

use log::{debug, info};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{common::request::DataRequest, connect_data_response};

use super::super::services::Services;

use tokio::io::Result;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthLinkRequest {
    pub tunnel_id: Uuid,
    pub session_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AuthLinkResponse {
    Accepted,
    Rejected { reason: String },
}

connect_data_response!(AuthLinkRequest, AuthLinkResponse);

pub async fn handle_auth_link(services: Arc<Services>, mut request: DataRequest<AuthLinkRequest>) {
    let config = services.get_config();
}
