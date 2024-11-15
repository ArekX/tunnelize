use serde::{Deserialize, Serialize};

use crate::{
    common::{
        connection::Connection,
        encryption::ClientEncryptionType,
        tcp_client::create_tcp_client,
        validate::{Validatable, Validation},
        validate_rules::{
            AlphaNumericOnly, FileMustExist, HostAddressMustBeValid, IpAddressMustBeValid,
            MustNotBeEmptyString, PortMustBeValid,
        },
    },
    configuration::TunnelizeConfiguration,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TunnelConfiguration {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    pub server_address: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_port: Option<u16>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub forward_connection_timeout_seconds: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub encryption: Option<TunnelEncryption>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tunnel_key: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub monitor_key: Option<String>,

    pub proxies: Vec<TunnelProxy>,
}

impl TunnelConfiguration {
    pub async fn create_tcp_client(&self) -> tokio::io::Result<Connection> {
        create_tcp_client(
            &self.server_address,
            self.get_server_port(),
            self.get_encryption().to_encryption_type(),
        )
        .await
    }

    pub fn get_server_port(&self) -> u16 {
        self.server_port.unwrap_or(3456)
    }

    pub fn get_forward_connection_timeout_seconds(&self) -> u64 {
        self.forward_connection_timeout_seconds.unwrap_or(30)
    }

    pub fn get_encryption(&self) -> TunnelEncryption {
        self.encryption.clone().unwrap_or(TunnelEncryption::None)
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
            result.validate_rule::<FileMustExist>("cert", &cert);
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
        #[serde(skip_serializing_if = "Option::is_none")]
        desired_name: Option<String>,
    },
    Tcp {
        #[serde(skip_serializing_if = "Option::is_none")]
        desired_port: Option<u16>,
    },
    Udp {
        #[serde(skip_serializing_if = "Option::is_none")]
        desired_port: Option<u16>,
        #[serde(skip_serializing_if = "Option::is_none")]
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
