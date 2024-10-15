use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    common::channel::OkResponse, create_enum_channel, server::incoming_requests::ProxySession,
};

use super::http::HttpEndpointInfo;

create_enum_channel!(EndpointRequest -> EndpointResponse, {
    RegisterProxyRequest -> RegisterProxyResponse,
    RemoveTunnelRequest -> OkResponse
});

#[derive(Clone, Debug)]
pub struct RegisterProxyRequest {
    pub tunnel_id: Uuid,
    pub proxy_sessions: Vec<ProxySession>,
}

#[derive(Clone, Debug)]
pub struct RegisterProxyResponse {
    pub proxy_info: HashMap<Uuid, EndpointInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EndpointInfo {
    Http(HttpEndpointInfo),
}

#[derive(Clone, Debug)]
pub struct RemoveTunnelRequest {
    pub tunnel_id: Uuid,
}
