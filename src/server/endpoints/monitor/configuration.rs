use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MonitorEndpointConfig {
    pub port: u16,
    pub is_secure: bool,
    pub address: Option<String>,
    pub authentication: MonitorAuthentication,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MonitorAuthentication {
    None,
    Basic { username: String, password: String },
    Bearer { token: String },
}

impl MonitorEndpointConfig {
    pub fn get_bind_address(&self) -> String {
        let address = self.address.clone().unwrap_or_else(|| format!("0.0.0.0"));

        format!("{}:{}", address, self.port)
    }
}
