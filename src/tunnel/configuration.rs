use serde::{Deserialize, Serialize};

use crate::{
    common::{
        connection::Connection,
        tcp_client::{create_tcp_client, ClientEncryption},
        validate::{Validatable, Validation},
        validate_rules::{
            AlphaNumericOnly, HostAddressMustBeValid, IpAddressMustBeValid, MustNotBeEmptyString,
            PortMustBeValid,
        },
    },
    configuration::TunnelizeConfiguration,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TunnelConfiguration {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub name: Option<String>,

    pub server_address: String,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub server_port: Option<u16>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub forward_connection_timeout_seconds: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub encryption: Option<ClientEncryption>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub tunnel_key: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub monitor_key: Option<String>,

    pub proxies: Vec<TunnelProxy>,
}

impl TunnelConfiguration {
    pub async fn create_tcp_client(&self) -> tokio::io::Result<Connection> {
        create_tcp_client(
            &self.server_address,
            self.get_server_port(),
            self.get_encryption(),
        )
        .await
    }

    pub fn get_server_port(&self) -> u16 {
        self.server_port.unwrap_or(3456)
    }

    pub fn get_forward_connection_timeout_seconds(&self) -> u64 {
        self.forward_connection_timeout_seconds.unwrap_or(30)
    }

    pub fn get_encryption(&self) -> ClientEncryption {
        self.encryption.clone().unwrap_or(ClientEncryption::None)
    }
}

impl Into<TunnelizeConfiguration> for TunnelConfiguration {
    fn into(self) -> TunnelizeConfiguration {
        TunnelizeConfiguration {
            server: None,
            tunnel: Some(self),
        }
    }
}

impl TryFrom<TunnelizeConfiguration> for TunnelConfiguration {
    type Error = &'static str;

    fn try_from(value: TunnelizeConfiguration) -> Result<Self, Self::Error> {
        match value.tunnel {
            Some(tunnel) => Ok(tunnel),
            None => Err("Tunnel configuration is required."),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TunnelProxy {
    pub endpoint_name: String,
    pub address: String,
    pub port: u16,
    pub endpoint_config: ProxyConfiguration,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ProxyConfiguration {
    Http {
        #[serde(skip_serializing_if = "Option::is_none", default)]
        desired_name: Option<String>,
    },
    Tcp {
        #[serde(skip_serializing_if = "Option::is_none", default)]
        desired_port: Option<u16>,
    },
    Udp {
        #[serde(skip_serializing_if = "Option::is_none", default)]
        desired_port: Option<u16>,
        #[serde(skip_serializing_if = "Option::is_none", default)]
        bind_address: Option<String>,
    },
}

impl ProxyConfiguration {
    pub fn get_type_string(&self) -> &'static str {
        match self {
            Self::Http { .. } => "http",
            Self::Tcp { .. } => "tcp",
            Self::Udp { .. } => "udp",
        }
    }
}

impl Validatable for TunnelProxy {
    fn validate(&self, result: &mut Validation) {
        if self.endpoint_name.is_empty() {
            result.add_error("Endpoint name is required.");
        }

        result.validate_rule::<HostAddressMustBeValid>("address", &self.address);
        result.validate_rule::<PortMustBeValid>("port", &self.port);

        result.validate_child("endpoint_config", &self.endpoint_config);
    }
}

impl Validatable for ProxyConfiguration {
    fn validate(&self, result: &mut Validation) {
        match self {
            Self::Http { desired_name } => {
                if let Some(name) = desired_name {
                    result.validate_rule::<MustNotBeEmptyString>("desired_name", name);
                    result.validate_rule::<AlphaNumericOnly>("desired_name", name);
                }
            }
            Self::Tcp { desired_port } => {
                if let Some(port) = desired_port {
                    result.validate_rule::<PortMustBeValid>("desired_port", port);
                }
            }
            Self::Udp {
                desired_port,
                bind_address,
            } => {
                if let Some(port) = desired_port {
                    result.validate_rule::<PortMustBeValid>("desired_port", port);
                }

                if let Some(address) = bind_address {
                    result.validate_rule::<IpAddressMustBeValid>("bind_address", address);
                }
            }
        }
    }
}

impl Validatable for TunnelConfiguration {
    fn validate(&self, result: &mut Validation) {
        result.validate_rule::<HostAddressMustBeValid>("server_address", &self.server_address);
        result.validate_rule::<PortMustBeValid>("server_port", &self.get_server_port());

        if self.get_forward_connection_timeout_seconds() == 0 {
            result.add_field_error(
                "forward_connection_timeout_seconds",
                "Forward connection timeout is required.",
            );
        }

        result.validate_child("encryption", &self.get_encryption());

        if self.proxies.len() == 0 {
            result.add_field_error("proxies", "At least one proxy is required.");
            return;
        }

        for (index, proxy) in self.proxies.iter().enumerate() {
            result.validate_child(&format!("proxies.{}", index), proxy);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_tunnel_configuration() -> TunnelConfiguration {
        TunnelConfiguration {
            name: Some("test_tunnel".to_string()),
            server_address: "127.0.0.1".to_string(),
            server_port: Some(8080),
            forward_connection_timeout_seconds: Some(60),
            encryption: Some(ClientEncryption::None),
            tunnel_key: Some("test_key".to_string()),
            monitor_key: Some("monitor_key".to_string()),
            proxies: vec![TunnelProxy {
                endpoint_name: "test_proxy".to_string(),
                address: "127.0.0.1".to_string(),
                port: 8081,
                endpoint_config: ProxyConfiguration::Http {
                    desired_name: Some("test_http".to_string()),
                },
            }],
        }
    }

    #[test]
    fn test_get_server_port() {
        let config = create_test_tunnel_configuration();
        assert_eq!(config.get_server_port(), 8080);

        let config = TunnelConfiguration {
            server_port: None,
            ..create_test_tunnel_configuration()
        };
        assert_eq!(config.get_server_port(), 3456);
    }

    #[test]
    fn test_get_forward_connection_timeout_seconds() {
        let config = create_test_tunnel_configuration();
        assert_eq!(config.get_forward_connection_timeout_seconds(), 60);

        let config = TunnelConfiguration {
            forward_connection_timeout_seconds: None,
            ..create_test_tunnel_configuration()
        };
        assert_eq!(config.get_forward_connection_timeout_seconds(), 30);
    }

    #[test]
    fn test_get_encryption() {
        let config = create_test_tunnel_configuration();
        assert_eq!(config.get_encryption(), ClientEncryption::None);

        let config = TunnelConfiguration {
            encryption: Some(ClientEncryption::Tls {
                ca_path: Some("path/to/cert".to_string()),
            }),
            ..create_test_tunnel_configuration()
        };
        assert_eq!(
            config.get_encryption(),
            ClientEncryption::Tls {
                ca_path: Some("path/to/cert".to_string()),
            }
        );
    }
}
