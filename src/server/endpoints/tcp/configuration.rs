use serde::{Deserialize, Serialize};

use crate::server::configuration::EndpointServerEncryption;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TcpEndpointConfig {
    pub address: Option<String>,
    pub allow_desired_port: bool,
    pub reserve_ports_from: u16,
    pub reserve_ports_to: u16,
    pub encryption: EndpointServerEncryption,
    pub full_hostname_template: Option<String>,
}

impl TcpEndpointConfig {
    pub fn get_bind_address(&self, port: u16) -> String {
        let address = self.address.clone().unwrap_or_else(|| format!("0.0.0.0"));

        format!("{}:{}", address, port)
    }

    pub fn get_address(&self) -> String {
        self.address.clone().unwrap_or_else(|| format!("0.0.0.0"))
    }

    pub fn get_assigned_hostname(&self, port: u16) -> String {
        if let Some(template) = &self.full_hostname_template {
            return template.replace("{port}", &port.to_string());
        }

        self.get_bind_address(port)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TcpPublicEndpointConfig {
    pub address: Option<String>,
    pub allow_desired_port: bool,
    pub reserve_ports_from: u16,
    pub reserve_ports_to: u16,
}

impl From<&TcpEndpointConfig> for TcpPublicEndpointConfig {
    fn from(config: &TcpEndpointConfig) -> Self {
        Self {
            address: config.address.clone(),
            allow_desired_port: config.allow_desired_port.clone(),
            reserve_ports_from: config.reserve_ports_from.clone(),
            reserve_ports_to: config.reserve_ports_to.clone(),
        }
    }
}
