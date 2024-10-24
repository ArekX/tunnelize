use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TcpEndpointConfig {
    pub is_secure: bool,
    pub address: Option<String>,
    pub reserve_ports_from: u16,
    pub reserve_ports_to: u16,
}

impl TcpEndpointConfig {
    pub fn get_bind_address(&self, port: u16) -> String {
        let address = self.address.clone().unwrap_or_else(|| format!("0.0.0.0"));

        format!("{}:{}", address, port)
    }
}
