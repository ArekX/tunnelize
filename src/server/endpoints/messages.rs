use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    common::channel::OkResponse, create_channel_enum, server::incoming_requests::ProxySession,
};

use super::{http::HttpEndpointInfo, tcp::TcpEndpointInfo};

create_channel_enum!(EndpointChannelRequest -> EndpointChannelResponse, {
    RegisterTunnelRequest -> RegisterTunnelResponse,
    RemoveTunnelRequest -> OkResponse
});

#[derive(Clone, Debug)]
pub struct RegisterTunnelRequest {
    pub tunnel_id: Uuid,
    pub proxy_sessions: Vec<ProxySession>,
}

#[derive(Clone, Debug)]
pub enum RegisterTunnelResponse {
    Accepted {
        proxy_info: HashMap<Uuid, ResolvedEndpointInfo>,
    },
    Rejected {
        reason: String,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResolvedEndpointInfo {
    Http(HttpEndpointInfo),
    Tcp(TcpEndpointInfo),
}

#[derive(Clone, Debug)]
pub struct RemoveTunnelRequest {
    pub tunnel_id: Uuid,
}
