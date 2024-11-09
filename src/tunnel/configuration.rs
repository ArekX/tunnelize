use std::fs::exists;

use serde::{Deserialize, Serialize};

use crate::{
    common::{
        connection::Connection,
        encryption::ClientEncryptionType,
        tcp_client::create_tcp_client,
        validate::{Rule, Validatable, Validation},
        validate_rules::{
            AlphaNumericOnly, FileMustExist, HostAddressMustBeValid, IpAddressMustBeValid,
            PortMustBeValid,
        },
    },
    configuration::TunnelizeConfiguration,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TunnelConfiguration {
    pub name: Option<String>,
    pub server_address: String,
    pub server_port: u16,
    pub forward_connection_timeout_seconds: u64,
    pub encryption: TunnelEncryption,
    pub tunnel_key: Option<String>,
    pub monitor_key: Option<String>,
    pub proxies: Vec<TunnelProxy>,
}

impl TunnelConfiguration {
    pub async fn create_tcp_client(&self) -> tokio::io::Result<Connection> {
        create_tcp_client(
            &self.server_address,
            self.server_port,
            self.encryption.to_encryption_type(),
        )
        .await
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
            None => Err("Server configuration is required."),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TunnelEncryption {
    None,
    Tls { cert: String },
    NativeTls,
}

impl TunnelEncryption {
    pub fn to_encryption_type(&self) -> Option<ClientEncryptionType> {
        match &self {
            TunnelEncryption::None => None,
            TunnelEncryption::Tls { cert } => Some(ClientEncryptionType::CustomTls {
                ca_cert_path: cert.clone(),
            }),
            TunnelEncryption::NativeTls => Some(ClientEncryptionType::NativeTls),
        }
    }
}

impl From<Option<ClientEncryptionType>> for TunnelEncryption {
    fn from(value: Option<ClientEncryptionType>) -> Self {
        match value {
            Some(ClientEncryptionType::CustomTls { ca_cert_path }) => {
                Self::Tls { cert: ca_cert_path }
            }
            Some(ClientEncryptionType::NativeTls) => Self::NativeTls,
            None => Self::None,
        }
    }
}

impl Validatable for TunnelEncryption {
    fn validate(&self, result: &mut Validation) {
        if let Self::Tls { cert } = self {
            result.validate_rule::<FileMustExist, String>("cert", &cert);
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
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProxyConfiguration {
    Http {
        desired_name: Option<String>,
    },
    Tcp {
        desired_port: Option<u16>,
    },
    Udp {
        desired_port: Option<u16>,
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

        if self.address.is_empty() {
            result.add_error("Address is required.");
        }

        result.validate_rule::<PortMustBeValid, u16>("port", &self.port);

        result.validate_child("endpoint_config", &self.endpoint_config);
    }
}

impl Validatable for ProxyConfiguration {
    fn validate(&self, result: &mut Validation) {
        match self {
            Self::Http { desired_name } => {
                if let Some(name) = desired_name {
                    result.validate_rule::<AlphaNumericOnly, String>("desired_name", name);
                }
            }
            Self::Tcp { desired_port } => {
                if let Some(port) = desired_port {
                    result.validate_rule::<PortMustBeValid, u16>("desired_port", port);
                }
            }
            Self::Udp {
                desired_port,
                bind_address,
            } => {
                if let Some(port) = desired_port {
                    result.validate_rule::<PortMustBeValid, u16>("desired_port", port);
                }

                if let Some(address) = bind_address {
                    result.validate_rule::<IpAddressMustBeValid, String>("bind_address", address);
                }
            }
        }
    }
}

impl Validatable for TunnelConfiguration {
    fn validate(&self, result: &mut Validation) {
        result.validate_rule::<PortMustBeValid, u16>("server_port", &self.server_port);

        if self.forward_connection_timeout_seconds == 0 {
            result.add_field_error(
                "forward_connection_timeout_seconds",
                "Forward connection timeout is required.",
            );
        }

        result.validate_child("encryption", &self.encryption);

        if self.proxies.len() == 0 {
            result.add_field_error("proxies", "At least one proxy is required.");
            return;
        }

        for (index, proxy) in self.proxies.iter().enumerate() {
            result.validate_child(&format!("proxies[{}]", index), proxy);
        }
        // TODO: Validate other
    }
}
