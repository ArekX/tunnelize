use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TcpEndpointConfig {
    pub port: u16,
    pub is_secure: bool,
    pub address: Option<String>,
}

impl TcpEndpointConfig {
    pub fn get_bind_address(&self) -> String {
        let address = self.address.clone().unwrap_or_else(|| format!("0.0.0.0"));

        format!("{}:{}", address, self.port)
    }
}
