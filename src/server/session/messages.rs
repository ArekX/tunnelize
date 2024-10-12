use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::server::endpoints::EndpointInfo;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TunnelSessionMessage {
    EndpointInfo(EndpointInfo),
    ClientLinkRequest { client_id: Uuid },
}
