use std::{
    io::{Error, ErrorKind},
    sync::Arc,
};

use log::{debug, info};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{common::data_request::DataRequest, connect_data_response};

use super::super::services::Services;

use tokio::io::Result;

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

pub async fn process_init_link(services: Arc<Services>, mut request: DataRequest<InitLinkRequest>) {
    let config = services.get_config();
}
