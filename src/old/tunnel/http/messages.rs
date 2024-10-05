use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::server::http::ClientAuthorizeUser;

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
