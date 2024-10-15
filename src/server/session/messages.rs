use uuid::Uuid;

use crate::create_enum_channel;

create_enum_channel!(TunnelSessionRequest -> TunnelSessionResponse, {
    ClientLinkRequest -> ClientLinkResponse
});

#[derive(Debug)]
pub struct ClientLinkRequest {
    pub client_id: Uuid,
    pub endpoint_name: String,
}

#[derive(Debug)]
pub enum ClientLinkResponse {
    Accepted,
    Rejected { reason: String },
}
