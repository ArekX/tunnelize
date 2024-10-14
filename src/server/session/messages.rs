use uuid::Uuid;

use crate::{
    common::channel::{DataResponse, RequestEnum},
    connect_struct_with_request_enum, connect_struct_with_response_enum,
    server::endpoints::EndpointInfo,
};

#[derive(Debug)]
pub enum TunnelSessionMessage {
    EndpointInfo(EndpointInfo),
    ClientLinkRequest(ClientLinkRequest),
}

impl RequestEnum for TunnelSessionMessage {
    type ResponseEnum = TunnelSessionResponse;
}

#[derive(Debug)]
pub enum TunnelSessionResponse {
    ClientLinkResponse(ClientLinkResponse),
}

#[derive(Debug)]
pub struct ClientLinkRequest {
    pub client_id: Uuid,
    pub endpoint_name: String,
}

connect_struct_with_request_enum!(ClientLinkRequest, TunnelSessionMessage);

#[derive(Debug)]
pub enum ClientLinkResponse {
    Accepted,
    Rejected { reason: String },
}

connect_struct_with_response_enum!(ClientLinkResponse, TunnelSessionResponse);

impl DataResponse for ClientLinkRequest {
    type Response = ClientLinkResponse;
}
