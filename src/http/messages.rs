use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum TunnelMessage {
    Connect {
        proxies: Vec<Proxy>,
    },
    Disconnect {
        tunnel_id: u32,
    },
    ClientLinkDeny {
        tunnel_id: u32,
        client_id: u32,
        reason: String,
    },
    ClientLinkAccept {
        tunnel_id: u32,
        client_id: u32,
    },
}

#[derive(Serialize, Deserialize)]
pub struct Proxy {
    pub desired_name: Option<String>,
    pub forward_address: String,
}

#[derive(Serialize, Deserialize)]
pub enum ServerMessage {
    TunnelAccept {
        tunnel_id: u32,
        resolved_links: Vec<ResolvedLink>,
    },
    ClientLinkRequest {
        client_id: u32,
        host_id: u32,
    },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ResolvedLink {
    pub forward_address: String,
    pub hostname: String,
    pub host_id: u32,
}
