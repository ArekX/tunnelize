use std::{
    collections::HashMap,
    fs::exists,
    io::{Error, ErrorKind},
    sync::Arc,
};

use serde::{Deserialize, Serialize};

use crate::{
    common::{
        configuration::ServerEncryption,
        validate::{Validatable, Validation},
    },
    configuration::TunnelizeConfiguration,
    tunnel::configuration::ProxyConfiguration,
};

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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerConfiguration {
    pub server_port: u16,
    pub server_address: Option<String>,
    pub max_tunnel_input_wait: u16,
    pub tunnel_key: Option<String>,
    pub monitor_key: Option<String>,
    pub endpoints: HashMap<String, EndpointConfiguration>,
    pub encryption: ServerEncryption,
    pub max_tunnels: usize,
    pub max_clients: usize,
    pub max_proxies_per_tunnel: usize,
}

impl Into<TunnelizeConfiguration> for ServerConfiguration {
    fn into(self) -> TunnelizeConfiguration {
        TunnelizeConfiguration {
            server: Some(self),
            tunnel: None,
        }
    }
}

impl TryFrom<TunnelizeConfiguration> for ServerConfiguration {
    type Error = &'static str;

    fn try_from(value: TunnelizeConfiguration) -> Result<Self, Self::Error> {
        match value.server {
            Some(server) => Ok(server),
            None => Err("Server configuration is required."),
        }
    }
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

impl Validatable for ServerConfiguration {
    fn validate(&self, result: &mut Validation) {
        if self.server_port == 0 {
            result.add_field_error(
                "server_pot",
                "Server port must be set and must be larger than 0.",
            );
        }

        if self.max_clients == 0 {
            result.add_field_error(
                "max_clients",
                "Max clients must be set and must be larger than 0.",
            );
        }

        if self.max_tunnels == 0 {
            result.add_field_error(
                "max_tunnels",
                "Max tunnels must be set and must be larger than 0.",
            );
        }

        if self.max_proxies_per_tunnel == 0 {
            result.add_field_error(
                "max_proxies_per_tunnel",
                "Max proxies per tunnel must be set and must be larger than 0.",
            );
        }

        self.encryption.validate(result);

        if let Some(key) = &self.tunnel_key {
            if key.is_empty() {
                result.add_field_error("tunnel_key", "Tunnel key must not be empty.");
            }
        }

        if let Some(key) = &self.monitor_key {
            if key.is_empty() {
                result.add_field_error("monitor_key", "Monitor key must not be empty.");
            }
        }

        for (name, endpoint) in &self.endpoints {
            result.validate_child(&format!("endpoints.{}", name), endpoint);
        }
    }
}

impl Validatable for EndpointConfiguration {
    fn validate(&self, result: &mut Validation) {
        match self {
            Self::Http(config) => config.validate(result),
            Self::Tcp(config) => config.validate(result),
            Self::Udp(config) => config.validate(result),
            Self::Monitoring(config) => config.validate(result),
        }
    }
}

impl Validatable for EndpointServerEncryption {
    fn validate(&self, result: &mut Validation) {
        if let EndpointServerEncryption::CustomTls {
            cert_path,
            key_path,
        } = self
        {
            if !exists(cert_path).is_ok() {
                result.add_error(&format!(
                    "TLS cert path '{}' does not exist or is invalid.",
                    cert_path
                ));
            }

            if !exists(key_path).is_ok() {
                result.add_error(&format!(
                    "TLS key path '{}' does not exist or is invalid.",
                    key_path
                ));
            }
        }
    }
}
