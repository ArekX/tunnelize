use uuid::Uuid;

use crate::create_channel_enum;

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
