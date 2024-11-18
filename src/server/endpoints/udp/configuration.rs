use std::fmt::Display;

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
    pub fn get_address(&self) -> String {
        self.address.clone().unwrap_or_else(|| format!("0.0.0.0"))
    }

    pub fn get_bind_address(&self, port: u16) -> String {
        format!("{}:{}", self.get_address(), port)
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
    pub address: String,
    pub allow_desired_port: bool,
    pub reserve_ports_from: u16,
    pub reserve_ports_to: u16,
}

impl From<&UdpEndpointConfig> for UdpPublicEndpointConfig {
    fn from(config: &UdpEndpointConfig) -> Self {
        Self {
            address: config.get_address(),
            allow_desired_port: config.get_allow_desired_port(),
            reserve_ports_from: config.reserve_ports_from.clone(),
            reserve_ports_to: config.reserve_ports_to.clone(),
        }
    }
}

impl Display for UdpPublicEndpointConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Address: {}", self.address)?;
        writeln!(
            f,
            "User can request port: {}",
            if self.allow_desired_port { "Yes" } else { "No" }
        )?;
        writeln!(
            f,
            "Port range: {} - {}",
            self.reserve_ports_from, self.reserve_ports_to
        )?;

        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    fn get_config() -> UdpEndpointConfig {
        UdpEndpointConfig {
            address: Some("127.0.0.1".to_string()),
            allow_desired_port: Some(true),
            inactivity_timeout: Some(300),
            reserve_ports_from: 1000,
            reserve_ports_to: 2000,
            full_hostname_template: Some("host:{port}".to_string()),
        }
    }

    #[test]
    fn test_get_bind_address() {
        let config = get_config();
        assert_eq!(config.get_bind_address(8080), "127.0.0.1:8080");
    }

    #[test]
    fn test_get_assigned_hostname_with_template() {
        let config = get_config();
        assert_eq!(config.get_assigned_hostname(8080), "host:8080");
    }

    #[test]
    fn test_get_assigned_hostname_without_template() {
        let mut config = get_config();
        config.full_hostname_template = None;
        assert_eq!(config.get_assigned_hostname(8080), "127.0.0.1:8080");
    }

    #[test]
    fn test_get_allow_desired_port() {
        let config = get_config();
        assert!(config.get_allow_desired_port());
    }

    #[test]
    fn test_get_allow_desired_port_default() {
        let mut config = get_config();
        config.allow_desired_port = None;
        assert!(config.get_allow_desired_port());
    }

    #[test]
    fn test_get_inactivity_timeout() {
        let config = get_config();
        assert_eq!(config.get_inactivity_timeout(), 300);
    }

    #[test]
    fn test_get_inactivity_timeout_default() {
        let mut config = get_config();
        config.inactivity_timeout = None;
        assert_eq!(config.get_inactivity_timeout(), 300);
    }

    #[test]
    fn test_get_address() {
        let config = get_config();
        assert_eq!(config.get_address(), "127.0.0.1");
    }

    #[test]
    fn test_get_address_default() {
        let mut config = get_config();
        config.address = None;
        assert_eq!(config.get_address(), "0.0.0.0");
    }

    #[test]
    fn test_udp_public_endpoint_config_from() {
        let config = get_config();
        let public_config: UdpPublicEndpointConfig = (&config).into();
        assert_eq!(public_config.address, "127.0.0.1".to_string());
        assert!(public_config.allow_desired_port);
        assert_eq!(public_config.reserve_ports_from, 1000);
        assert_eq!(public_config.reserve_ports_to, 2000);
    }

    #[test]
    fn test_validate() {
        let config = get_config();
        let mut validation = Validation::new();
        config.validate(&mut validation);
        assert!(validation.is_valid());
    }

    #[test]
    fn test_validate_invalid_address() {
        let mut config = get_config();
        config.address = Some("invalid_address".to_string());
        let mut validation = Validation::new();
        config.validate(&mut validation);
        assert!(!validation.is_valid());
    }

    #[test]
    fn test_validate_invalid_ports() {
        let mut config = get_config();
        config.reserve_ports_from = 3000;
        config.reserve_ports_to = 2000;
        let mut validation = Validation::new();
        config.validate(&mut validation);
        assert!(!validation.is_valid());
    }
}
