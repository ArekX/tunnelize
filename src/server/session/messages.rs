use uuid::Uuid;

use crate::{
    common::channel_request::ChannelRequest, connect_channel_response, map_request_enum,
    server::endpoints::EndpointInfo,
};

#[derive(Debug)]
pub enum TunnelSessionMessage {
    EndpointInfo(EndpointInfo),
    ClientLink(ChannelRequest<ClientLinkRequest>),
}

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

connect_channel_response!(ClientLinkRequest, ClientLinkResponse);

map_request_enum!(TunnelSessionMessage, {
    ClientLinkRequest => ClientLink
});
