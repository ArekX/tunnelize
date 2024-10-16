use uuid::Uuid;

use crate::create_enum_channel;

create_enum_channel!(TunnelSessionRequest -> TunnelSessionResponse, {
    ClientLinkRequest -> ClientLinkResponse
});

#[derive(Debug)]
pub struct ClientLinkRequest {
    pub client_name: String,
    pub proxy_id: Uuid,
}

#[derive(Debug)]
pub enum ClientLinkResponse {
    Accepted,
    Rejected { reason: String },
}
