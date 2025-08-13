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
    http::configuration::{HttpEndpointConfig, HttpPublicEndpointConfig},
    monitor::configuration::{MonitorEndpointConfig, PublicMonitorEndpointConfig},
    tcp::configuration::{TcpEndpointConfig, TcpPublicEndpointConfig},
    udp::configuration::{UdpEndpointConfig, UdpPublicEndpointConfig},
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum EndpointServerEncryption {
    None,
    Tls {
        cert_path: Option<String>,
        key_path: Option<String>,
    },
}

impl EndpointServerEncryption {
    pub fn to_encryption(
        &self,
        server_config: &Arc<ServerConfiguration>,
    ) -> tokio::io::Result<ServerEncryption> {
        match self {
            EndpointServerEncryption::None => Ok(ServerEncryption::None),
            EndpointServerEncryption::Tls {
                cert_path: Some(cert_path),
                key_path: Some(key_path),
            } => Ok(ServerEncryption::Tls {
                cert_path: cert_path.clone(),
                key_path: key_path.clone(),
            }),
            EndpointServerEncryption::Tls {
                cert_path: None,
                key_path: None,
            } => {
                let (cert_path, key_path) = match server_config.encryption {
                    Some(ServerEncryption::Tls {
                        ref cert_path,
                        ref key_path,
                    }) => (cert_path, key_path),
                    _ => {
                        return Err(Error::new(
                            ErrorKind::InvalidInput,
                            "Main server TLS encryption is not set, but is required".to_string(),
                        ));
                    }
                };

                Ok(ServerEncryption::Tls {
                    cert_path: cert_path.clone(),
                    key_path: key_path.clone(),
                })
            }
            _ => Err(Error::new(
                ErrorKind::InvalidInput,
                "Both cert_path and key_path must be set for custom TLS certificate",
            )),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerConfiguration {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub server_port: Option<u16>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub server_address: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub max_tunnel_input_wait: Option<u16>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub tunnel_key: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub monitor_key: Option<String>,

    pub endpoints: HashMap<String, EndpointConfiguration>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub encryption: Option<ServerEncryption>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub max_tunnels: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub max_clients: Option<usize>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub max_proxies_per_tunnel: Option<usize>,
}

impl From<ServerConfiguration> for TunnelizeConfiguration {
    fn from(val: ServerConfiguration) -> Self {
        TunnelizeConfiguration {
            server: Some(val),
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
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum EndpointConfiguration {
    Http(HttpEndpointConfig),
    Tcp(TcpEndpointConfig),
    Udp(UdpEndpointConfig),
    Monitoring(MonitorEndpointConfig),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PublicEndpointConfiguration {
    Http(HttpPublicEndpointConfig),
    Tcp(TcpPublicEndpointConfig),
    Udp(UdpPublicEndpointConfig),
    Monitoring(PublicMonitorEndpointConfig),
}

impl From<&EndpointConfiguration> for PublicEndpointConfiguration {
    fn from(config: &EndpointConfiguration) -> Self {
        match config {
            EndpointConfiguration::Http(config) => PublicEndpointConfiguration::Http(config.into()),
            EndpointConfiguration::Tcp(config) => PublicEndpointConfiguration::Tcp(config.into()),
            EndpointConfiguration::Udp(config) => PublicEndpointConfiguration::Udp(config.into()),
            EndpointConfiguration::Monitoring(config) => {
                PublicEndpointConfiguration::Monitoring(config.into())
            }
        }
    }
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
            result.validate_child(&format!("endpoints.{name}"), endpoint);
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
        match self {
            EndpointServerEncryption::Tls {
                cert_path: Some(cert_path),
                key_path: Some(key_path),
            } => {
                result.validate_rule::<FileMustExist>("cert_path", cert_path);
                result.validate_rule::<FileMustExist>("key_path", key_path);
            }
            Self::None => {}
            _ => {
                result.add_error(
                    "Both cert_path and key_path must be set for custom TLS certificate",
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn default_server_config() -> ServerConfiguration {
        ServerConfiguration {
            server_port: Some(8080),
            server_address: Some("127.0.0.1".to_string()),
            max_tunnel_input_wait: Some(60),
            tunnel_key: Some("tunnel_key".to_string()),
            monitor_key: Some("monitor_key".to_string()),
            endpoints: HashMap::new(),
            encryption: Some(ServerEncryption::None),
            max_tunnels: Some(200),
            max_clients: Some(200),
            max_proxies_per_tunnel: Some(20),
        }
    }

    #[test]
    fn test_get_server_address() {
        let config = default_server_config();
        assert_eq!(config.get_server_address(), "127.0.0.1");

        let config = ServerConfiguration {
            server_address: None,
            ..default_server_config()
        };
        assert_eq!(config.get_server_address(), "0.0.0.0");
    }

    #[test]
    fn test_get_server_port() {
        let config = default_server_config();
        assert_eq!(config.get_server_port(), 8080);

        let config = ServerConfiguration {
            server_port: None,
            ..default_server_config()
        };
        assert_eq!(config.get_server_port(), 3456);
    }

    #[test]
    fn test_get_max_tunnel_input_wait() {
        let config = default_server_config();
        assert_eq!(config.get_max_tunnel_input_wait(), 60);

        let config = ServerConfiguration {
            max_tunnel_input_wait: None,
            ..default_server_config()
        };
        assert_eq!(config.get_max_tunnel_input_wait(), 30);
    }

    #[test]
    fn test_get_max_tunnels() {
        let config = default_server_config();
        assert_eq!(config.get_max_tunnnels(), 200);

        let config = ServerConfiguration {
            max_tunnels: None,
            ..default_server_config()
        };
        assert_eq!(config.get_max_tunnnels(), 100);
    }

    #[test]
    fn test_get_max_clients() {
        let config = default_server_config();
        assert_eq!(config.get_max_clients(), 200);

        let config = ServerConfiguration {
            max_clients: None,
            ..default_server_config()
        };
        assert_eq!(config.get_max_clients(), 100);
    }

    #[test]
    fn test_get_max_proxies_per_tunnel() {
        let config = default_server_config();
        assert_eq!(config.get_max_proxies_per_tunnel(), 20);

        let config = ServerConfiguration {
            max_proxies_per_tunnel: None,
            ..default_server_config()
        };
        assert_eq!(config.get_max_proxies_per_tunnel(), 10);
    }

    #[test]
    fn test_get_encryption() {
        let config = default_server_config();
        assert_eq!(config.get_encryption(), ServerEncryption::None);

        let config = ServerConfiguration {
            encryption: None,
            ..default_server_config()
        };
        assert_eq!(config.get_encryption(), ServerEncryption::None);
    }

    #[test]
    fn test_endpoint_server_encryption_to_encryption() {
        let server_config = Arc::new(default_server_config());

        let encryption = EndpointServerEncryption::None;
        assert_eq!(
            encryption.to_encryption(&server_config).unwrap(),
            ServerEncryption::None
        );

        let encryption = EndpointServerEncryption::Tls {
            cert_path: Some("cert.pem".to_string()),
            key_path: Some("key.pem".to_string()),
        };
        assert_eq!(
            encryption.to_encryption(&server_config).unwrap(),
            ServerEncryption::Tls {
                cert_path: "cert.pem".to_string(),
                key_path: "key.pem".to_string(),
            }
        );

        let encryption = EndpointServerEncryption::Tls {
            cert_path: None,
            key_path: None,
        };
        let server_config_with_tls = Arc::new(ServerConfiguration {
            encryption: Some(ServerEncryption::Tls {
                cert_path: "default_cert.pem".to_string(),
                key_path: "default_key.pem".to_string(),
            }),
            ..default_server_config()
        });
        assert_eq!(
            encryption.to_encryption(&server_config_with_tls).unwrap(),
            ServerEncryption::Tls {
                cert_path: "default_cert.pem".to_string(),
                key_path: "default_key.pem".to_string(),
            }
        );
    }
}
