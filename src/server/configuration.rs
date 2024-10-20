use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::tunnel::configuration::ProxyConfiguration;

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
    // TODO: Udp { port_range: (u16, u16) },
    // TODO: MonitoringApi { port: u16 },
}

impl EndpointConfiguration {
    pub fn matches_proxy_type(&self, proxy: &ProxyConfiguration) -> bool {
        match (self, proxy) {
            (Self::Http(_), ProxyConfiguration::Http { .. }) => true,
            (Self::Tcp(_), &ProxyConfiguration::Tcp { .. }) => true,
            // TODO: (Self::Udp(_), &ProxyConfiguration::Udp { .. }) => true,
            _ => false,
        }
    }

    pub fn get_type_string(&self) -> &'static str {
        match self {
            Self::Http(_) => "Http",
            Self::Tcp(_) => "Tcp",
            // TODO: Others
        }
    }
}
