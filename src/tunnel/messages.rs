use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ProxyMessage {}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TunnelRequestMessage {
    RequestLinkSession {
        service_name: String,
        session_key: Uuid,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TunnelResponseMessage {
    LinkSessionAccepted,
    LinkSessionRejected { reason: String },
}
