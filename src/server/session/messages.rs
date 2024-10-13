use uuid::Uuid;

use crate::{
    common::channel::{DataResponse, RequestEnum},
    connect_request_struct_with_enum, connect_response_struct_with_enum,
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

connect_request_struct_with_enum!(ClientLinkRequest, TunnelSessionMessage);

#[derive(Debug)]
pub enum ClientLinkResponse {
    Accepted,
    Rejected { reason: String },
}

connect_response_struct_with_enum!(ClientLinkResponse, TunnelSessionResponse);

impl DataResponse for ClientLinkRequest {
    type Response = ClientLinkResponse;
}
