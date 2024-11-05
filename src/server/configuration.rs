use std::{
    collections::HashMap,
    io::{Error, ErrorKind},
    sync::Arc,
};

use serde::{Deserialize, Serialize};

use crate::{common::tcp_server::ServerEncryption, tunnel::configuration::ProxyConfiguration};

use super::endpoints::{
    http::configuration::HttpEndpointConfig, monitor::configuration::MonitorEndpointConfig,
    tcp::configuration::TcpEndpointConfig, udp::configuration::UdpEndpointConfig,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EndpointServerEncryption {
    None,
    CustomTls { cert_path: String, key_path: String },
    ServerTls,
}

impl EndpointServerEncryption {
    pub fn to_encryption(
        &self,
        server_config: &Arc<ServerConfiguration>,
    ) -> tokio::io::Result<ServerEncryption> {
        match self {
            EndpointServerEncryption::None => Ok(ServerEncryption::None),
            EndpointServerEncryption::CustomTls {
                cert_path,
                key_path,
            } => Ok(ServerEncryption::Tls {
                cert_path: cert_path.clone(),
                key_path: key_path.clone(),
            }),
            EndpointServerEncryption::ServerTls => {
                let (cert_path, key_path) = match server_config.encryption {
                    ServerEncryption::Tls {
                        ref cert_path,
                        ref key_path,
                    } => (cert_path, key_path),
                    ServerEncryption::None => {
                        return Err(Error::new(
                            ErrorKind::InvalidInput,
                            format!("Tunnel server TLS encryption is not set, but is required"),
                        ));
                    }
                };

                Ok(ServerEncryption::Tls {
                    cert_path: cert_path.clone(),
                    key_path: key_path.clone(),
                })
            }
        }
    }
}

// Set max tunnels and clients.

#[derive(Serialize, Deserialize)]
pub struct ServerConfiguration {
    pub server_port: u16,
    pub server_address: Option<String>,
    pub max_tunnel_input_wait: u16,
    pub tunnel_key: Option<String>,
    pub monitor_key: Option<String>,
    pub endpoints: HashMap<String, EndpointConfiguration>,
    pub encryption: ServerEncryption,
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
