use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UdpEndpointConfig {
    pub address: Option<String>,
    pub allow_desired_port: bool,
    pub inactivity_timeout: u64,
    pub reserve_ports_from: u16,
    pub reserve_ports_to: u16,
    pub full_hostname_template: Option<String>,
}

impl UdpEndpointConfig {
    pub fn get_bind_address(&self, port: u16) -> String {
        let address = self.address.clone().unwrap_or_else(|| format!("0.0.0.0"));

        format!("{}:{}", address, port)
    }

    pub fn get_assigned_hostname(&self, port: u16) -> String {
        if let Some(template) = &self.full_hostname_template {
            return template.replace("{port}", &port.to_string());
        }

        self.get_bind_address(port)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UdpPublicEndpointConfig {
    pub address: Option<String>,
    pub allow_desired_port: bool,
    pub reserve_ports_from: u16,
    pub reserve_ports_to: u16,
}

impl From<&UdpEndpointConfig> for UdpPublicEndpointConfig {
    fn from(config: &UdpEndpointConfig) -> Self {
        Self {
            address: config.address.clone(),
            allow_desired_port: config.allow_desired_port.clone(),
            reserve_ports_from: config.reserve_ports_from.clone(),
            reserve_ports_to: config.reserve_ports_to.clone(),
        }
    }
}
