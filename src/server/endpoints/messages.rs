use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    common::channel::OkResponse, create_channel_enum, server::incoming_requests::ProxySession,
};

use super::http::HttpEndpointInfo;

create_channel_enum!(EndpointChannelRequest -> EndpointChannelResponse, {
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
    pub proxy_info: HashMap<Uuid, ResolvedEndpointInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResolvedEndpointInfo {
    Http(HttpEndpointInfo),
}

#[derive(Clone, Debug)]
pub struct RemoveTunnelRequest {
    pub tunnel_id: Uuid,
}
