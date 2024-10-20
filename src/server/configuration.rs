use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::endpoints::{http::HttpEndpointConfig, tcp::configuration::TcpEndpointConfig};

#[derive(Serialize, Deserialize)]
pub struct ServerConfiguration {
    pub server_port: u16,
    pub max_tunnel_input_wait: u16,
    pub endpoint_key: Option<String>,
    pub admin_key: Option<String>,
    pub endpoints: HashMap<String, EndpointConfiguration>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum EndpointConfiguration {
    Http(HttpEndpointConfig),
    Tcp(TcpEndpointConfig),
    // Tcp { port_range: (u16, u16) },
    // Udp { port_range: (u16, u16) },
    // MonitoringApi { port: u16 },
}
