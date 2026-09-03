use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{create_channel_enum, server::incoming_requests::HeartbeatRequest};

create_channel_enum!(TunnelChannelRequest -> TunnelChannelResponse, {
    ClientLinkRequest -> ClientLinkResponse
});

#[derive(Debug)]
pub struct ClientLinkRequest {
    pub client_id: Uuid,
    pub proxy_id: Uuid,
}

#[derive(Debug)]
pub enum ClientLinkResponse {
    Accepted,
    Rejected { reason: String },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TunnelSessionMessage {
    HeartbeatRequest(HeartbeatRequest),
    LinkRejected(LinkRejected),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LinkRejected {
    pub session_id: Uuid,
    pub reason: String,
}
