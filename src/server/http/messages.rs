use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::ClientAuthorizeUser;

#[derive(Serialize, Deserialize)]
pub enum ServerMessage {
    TunnelDeny {
        reason: String,
    },
    TunnelAccept {
        // TODO: Make sure that if this is already accepted it cannot be accepted twice
        tunnel_id: Uuid,
        resolved_links: Vec<ResolvedLink>,
    },
    ClientLinkRequest {
        client_id: Uuid,
        host_id: Uuid,
    },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ResolvedLink {
    pub forward_address: String,
    pub hostname: String,
    pub url: String,
    pub host_id: Uuid,
}
