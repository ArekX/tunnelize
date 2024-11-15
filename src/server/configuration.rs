use std::{
    collections::HashMap,
    io::{Error, ErrorKind},
    sync::Arc,
};

use serde::{Deserialize, Serialize};

use crate::{
    common::{
        configuration::ServerEncryption,
        validate::{Validatable, Validation},
        validate_rules::{
            FileMustExist, HostAddressMustBeValid, MustBeGreaterThanZero, MustNotBeEmptyString,
            PortMustBeValid,
        },
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
    Tls,
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
            EndpointServerEncryption::Tls => {
                let (cert_path, key_path) = match server_config.encryption {
                    Some(ServerEncryption::Tls {
                        ref cert_path,
                        ref key_path,
                    }) => (cert_path, key_path),
                    _ => {
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_port: Option<u16>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_address: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tunnel_input_wait: Option<u16>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tunnel_key: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub monitor_key: Option<String>,

    pub endpoints: HashMap<String, EndpointConfiguration>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub encryption: Option<ServerEncryption>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tunnels: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_clients: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_proxies_per_tunnel: Option<usize>,
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

    pub fn get_server_port(&self) -> u16 {
        self.server_port.unwrap_or(3456)
    }

    pub fn get_max_tunnel_input_wait(&self) -> u16 {
        self.max_tunnel_input_wait.unwrap_or(30)
    }

    pub fn get_max_tunnnels(&self) -> usize {
        self.max_tunnels.unwrap_or(100)
    }

    pub fn get_max_clients(&self) -> usize {
        self.max_clients.unwrap_or(100)
    }

    pub fn get_max_proxies_per_tunnel(&self) -> usize {
        self.max_proxies_per_tunnel.unwrap_or(10)
    }

    pub fn get_encryption(&self) -> ServerEncryption {
        self.encryption.clone().unwrap_or(ServerEncryption::None)
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
        result.validate_rule::<PortMustBeValid>("server_port", &self.get_server_port());

        if let Some(address) = &self.server_address {
            result.validate_rule::<HostAddressMustBeValid>("server_address", address);
        }

        result.validate_rule_for::<_, MustBeGreaterThanZero>(
            "max_tunnel_input_wait",
            &self.get_max_tunnel_input_wait(),
        );

        if let Some(key) = &self.tunnel_key {
            result.validate_rule::<MustNotBeEmptyString>("tunnel_key", key);
        }

        if let Some(key) = &self.monitor_key {
            result.validate_rule::<MustNotBeEmptyString>("monitor_key", key);
        }

        for (name, endpoint) in &self.endpoints {
            result.validate_child(&format!("endpoints.{}", name), endpoint);
        }

        result.validate_child("encryption", &self.get_encryption());

        result
            .validate_rule_for::<_, MustBeGreaterThanZero>("max_tunnels", &self.get_max_tunnnels());
        result
            .validate_rule_for::<_, MustBeGreaterThanZero>("max_clients", &self.get_max_clients());

        result.validate_rule_for::<_, MustBeGreaterThanZero>(
            "max_proxies_per_tunnel",
            &self.get_max_proxies_per_tunnel(),
        );
    }
}

impl Validatable for EndpointConfiguration {
    fn validate(&self, result: &mut Validation) {
        match self {
            Self::Http(config) => result.validate_child("config", config),
            Self::Tcp(config) => result.validate_child("config", config),
            Self::Udp(config) => result.validate_child("config", config),
            Self::Monitoring(config) => result.validate_child("config", config),
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
            result.validate_rule::<FileMustExist>("cert_path", cert_path);
            result.validate_rule::<FileMustExist>("key_path", key_path);
        }
    }
}
