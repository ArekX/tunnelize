use uuid::Uuid;

use crate::{
    common::channel::RequestEnum, connect_request_with_enum, connect_request_with_reponse_enum,
    connect_request_with_response_struct, connect_response_with_enum,
    server::endpoints::EndpointInfo,
};

#[derive(Debug)]
pub enum TunnelSessionMessage {
    EndpointInfo(EndpointInfo),
    ClientLinkRequest(ClientLinkRequest),
}

connect_request_with_reponse_enum!(TunnelSessionMessage, TunnelSessionResponse);

#[derive(Debug)]
pub enum TunnelSessionResponse {
    ClientLinkResponse(ClientLinkResponse),
}

#[derive(Debug)]
pub struct ClientLinkRequest {
    pub client_id: Uuid,
    pub endpoint_name: String,
}

connect_request_with_enum!(ClientLinkRequest, TunnelSessionMessage);
connect_request_with_response_struct!(ClientLinkRequest, ClientLinkResponse);

#[derive(Debug)]
pub enum ClientLinkResponse {
    Accepted,
    Rejected { reason: String },
}

connect_response_with_enum!(ClientLinkResponse, TunnelSessionResponse);
