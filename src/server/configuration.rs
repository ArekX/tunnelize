use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::tunnel::configuration::ProxyConfiguration;

use super::endpoints::{
    http::HttpEndpointConfig, monitor::configuration::MonitorEndpointConfig,
    tcp::configuration::TcpEndpointConfig, udp::configuration::UdpEndpointConfig,
};

#[derive(Serialize, Deserialize)]
pub struct ServerConfiguration {
    pub server_port: u16,
    pub server_address: Option<String>,
    pub max_tunnel_input_wait: u16,
    pub endpoint_key: Option<String>,
    pub monitor_key: Option<String>,
    pub endpoints: HashMap<String, EndpointConfiguration>,
}

impl ServerConfiguration {
    pub fn get_server_address(&self) -> String {
        self.server_address
            .as_ref()
            .map(|s| s.to_owned())
            .unwrap_or_else(|| "0.0.0.0".to_owned())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EndpointConfiguration {
    Http(HttpEndpointConfig),
    Tcp(TcpEndpointConfig),
    Udp(UdpEndpointConfig),
    Monitoring(MonitorEndpointConfig),
}

impl EndpointConfiguration {
    pub fn matches_proxy_type(&self, proxy: &ProxyConfiguration) -> bool {
        match (self, proxy) {
            (Self::Http(_), ProxyConfiguration::Http { .. }) => true,
            (Self::Tcp(_), &ProxyConfiguration::Tcp { .. }) => true,
            (Self::Udp(_), &ProxyConfiguration::Udp { .. }) => true,
            _ => false,
        }
    }

    pub fn get_type_string(&self) -> &'static str {
        match self {
            Self::Http(_) => "http",
            Self::Tcp(_) => "tcp",
            Self::Udp(_) => "udp",
            Self::Monitoring(_) => "monitoring",
        }
    }
}
