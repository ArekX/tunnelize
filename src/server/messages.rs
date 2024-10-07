use serde::{Deserialize, Serialize};

use crate::tunnel::configuration::ProxyConfiguration;

#[derive(Debug)]
pub enum ChannelMessage {}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ServerRequestMessage {
    AuthTunnelRequest {
        endpoint_key: Option<String>,
        admin_key: Option<String>,
        proxies: Vec<ProxyConfiguration>,
    },
    AuthLinkRequest {
        tunnel_id: String,
        session_key: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ServerResponseMessage {
    AuthTunnelAccepted { tunnel_id: String },
    AuthTunnelRejected { reason: String },
    AuthLinkAccepted,
    AuthLinkRejected { reason: String },
}
