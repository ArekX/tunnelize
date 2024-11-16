use serde::{Deserialize, Serialize};

use crate::common::{
    validate::{Validatable, Validation},
    validate_rules::{
        HostAddressMustBeValid, MustBeGreaterThanZero, PortHostnameTemplatemustBeValid,
        PortMustBeValid,
    },
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UdpEndpointConfig {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub address: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub allow_desired_port: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub inactivity_timeout: Option<u64>,

    pub reserve_ports_from: u16,
    pub reserve_ports_to: u16,

    #[serde(skip_serializing_if = "Option::is_none", default)]
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

    pub fn get_allow_desired_port(&self) -> bool {
        self.allow_desired_port.clone().unwrap_or_else(|| true)
    }

    pub fn get_inactivity_timeout(&self) -> u64 {
        self.inactivity_timeout.clone().unwrap_or_else(|| 300)
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
            allow_desired_port: config.get_allow_desired_port(),
            reserve_ports_from: config.reserve_ports_from.clone(),
            reserve_ports_to: config.reserve_ports_to.clone(),
        }
    }
}

impl Validatable for UdpEndpointConfig {
    fn validate(&self, result: &mut Validation) {
        if let Some(address) = &self.address {
            result.validate_rule::<HostAddressMustBeValid>("address", address);
        }

        if let Some(template) = &self.full_hostname_template {
            result.validate_rule::<PortHostnameTemplatemustBeValid>(
                "full_hostname_template",
                template,
            );
        }

        result.validate_rule_for::<_, MustBeGreaterThanZero>(
            "inactivity_timeout",
            &self.get_inactivity_timeout(),
        );

        if self.reserve_ports_from > self.reserve_ports_to {
            result.validate_rule::<PortMustBeValid>("reserve_ports_from", &self.reserve_ports_from);
            result.validate_rule::<PortMustBeValid>("reserve_ports_to", &self.reserve_ports_to);

            result.add_field_error(
                "reserve_ports_from",
                "reserve_ports_from must be less than reserve_ports_to.",
            );
        }
    }
}
