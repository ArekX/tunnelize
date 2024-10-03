use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::ClientAuthorizeUser;

#[derive(Serialize, Deserialize)]
pub enum HttpTunnelMessage {
    Connect {
        proxies: Vec<Proxy>,
        tunnel_auth_key: Option<String>,
        client_authorization: Option<ClientAuthorizeUser>,
    },
    Disconnect {
        tunnel_id: Uuid,
    },
    ClientLinkDeny {
        tunnel_id: Uuid,
        client_id: Uuid,
        reason: String,
    },
    ClientLinkAccept {
        tunnel_id: Uuid,
        client_id: Uuid,
    },
}

#[derive(Serialize, Deserialize)]
pub struct Proxy {
    pub desired_name: Option<String>,
    pub forward_address: String,
}

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
