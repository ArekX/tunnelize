use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;
use uuid::Uuid;

use crate::server::endpoints::EndpointInfo;

#[derive(Debug)]
pub enum TunnelSessionMessage {
    EndpointInfo(EndpointInfo),
    ClientLinkRequest {
        client_id: Uuid,
        endpoint_name: String,
        response_tx: oneshot::Sender<std::result::Result<(), String>>,
    },
}
