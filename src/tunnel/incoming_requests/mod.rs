use serde::{Deserialize, Serialize};

use crate::server::incoming_requests::HeartbeatResponse;

mod init_link_session;

pub use init_link_session::{InitLinkRequest, process_init_link};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TunnelRequestMessage {
    InitLinkRequest(InitLinkRequest),
    HeartbeatResponse(HeartbeatResponse),
}
