use serde::{Deserialize, Serialize};

use crate::server::configuration::EndpointServerEncryption;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MonitorEndpointConfig {
    pub port: u16,
    pub encryption: EndpointServerEncryption,
    pub address: Option<String>,
    pub authentication: MonitorAuthentication,
    pub allow_cors_origins: MonitorOrigin,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MonitorOrigin {
    Any,
    List(Vec<String>),
    None,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MonitorAuthentication {
    Basic { username: String, password: String },
    Bearer { token: String },
}

impl MonitorEndpointConfig {
    pub fn get_bind_address(&self) -> String {
        let address = self.address.clone().unwrap_or_else(|| format!("0.0.0.0"));

        format!("{}:{}", address, self.port)
    }
}
