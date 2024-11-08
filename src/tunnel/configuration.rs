use serde::{Deserialize, Serialize};

use crate::{
    common::{
        connection::Connection, encryption::ClientEncryptionType, tcp_client::create_tcp_client,
    },
    configuration::TunnelizeConfiguration,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TunnelConfiguration {
    pub name: Option<String>,
    pub server_address: String,
    pub server_port: u16,
    pub forward_connection_timeout_seconds: u64,
    pub encryption: Encryption,
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
pub enum Encryption {
    None,
    Tls { cert: String },
    NativeTls,
}

impl Encryption {
    pub fn to_encryption_type(&self) -> Option<ClientEncryptionType> {
        match &self {
            Encryption::None => None,
            Encryption::Tls { cert } => Some(ClientEncryptionType::CustomTls {
                ca_cert_path: cert.clone(),
            }),
            Encryption::NativeTls => Some(ClientEncryptionType::NativeTls),
        }
    }
}

impl From<Option<ClientEncryptionType>> for Encryption {
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
