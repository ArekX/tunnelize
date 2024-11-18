use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{
    common::{
        validate::{Validatable, Validation},
        validate_rules::{
            HostAddressMustBeValid, PortHostnameTemplatemustBeValid, PortMustBeValid,
        },
    },
    server::configuration::EndpointServerEncryption,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TcpEndpointConfig {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub address: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub allow_desired_port: Option<bool>,

    pub reserve_ports_from: u16,
    pub reserve_ports_to: u16,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub encryption: Option<EndpointServerEncryption>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
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

    pub fn get_allow_desired_port(&self) -> bool {
        self.allow_desired_port.clone().unwrap_or_else(|| true)
    }

    pub fn get_encryption(&self) -> EndpointServerEncryption {
        self.encryption
            .clone()
            .unwrap_or_else(|| EndpointServerEncryption::None)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TcpPublicEndpointConfig {
    pub address: String,
    pub allow_desired_port: bool,
    pub reserve_ports_from: u16,
    pub reserve_ports_to: u16,
}

impl From<&TcpEndpointConfig> for TcpPublicEndpointConfig {
    fn from(config: &TcpEndpointConfig) -> Self {
        Self {
            address: config.get_address(),
            allow_desired_port: config.get_allow_desired_port(),
            reserve_ports_from: config.reserve_ports_from.clone(),
            reserve_ports_to: config.reserve_ports_to.clone(),
        }
    }
}

impl Display for TcpPublicEndpointConfig {
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

impl Validatable for TcpEndpointConfig {
    fn validate(&self, result: &mut Validation) {
        if let Some(address) = &self.address {
            result.validate_rule::<HostAddressMustBeValid>("address", address);
        }

        result.validate_child("encryption", &self.get_encryption());

        if let Some(template) = &self.full_hostname_template {
            result.validate_rule::<PortHostnameTemplatemustBeValid>(
                "full_hostname_template",
                template,
            );
        }

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
    use crate::server::configuration::EndpointServerEncryption;

    fn create_config(
        address: Option<&str>,
        allow_desired_port: Option<bool>,
        encryption: Option<EndpointServerEncryption>,
        full_hostname_template: Option<&str>,
    ) -> TcpEndpointConfig {
        TcpEndpointConfig {
            address: address.map(|s| s.to_string()),
            allow_desired_port,
            reserve_ports_from: 8000,
            reserve_ports_to: 9000,
            encryption,
            full_hostname_template: full_hostname_template.map(|s| s.to_string()),
        }
    }

    #[test]
    fn test_get_bind_address() {
        let config = create_config(Some("127.0.0.1"), None, None, None);
        assert_eq!(config.get_bind_address(8080), "127.0.0.1:8080");

        let config = create_config(None, None, None, None);
        assert_eq!(config.get_bind_address(8080), "0.0.0.0:8080");
    }

    #[test]
    fn test_get_address() {
        let config = create_config(Some("127.0.0.1"), None, None, None);
        assert_eq!(config.get_address(), "127.0.0.1");

        let config = create_config(None, None, None, None);
        assert_eq!(config.get_address(), "0.0.0.0");
    }

    #[test]
    fn test_get_assigned_hostname() {
        let config = create_config(None, None, None, Some("host-{port}"));
        assert_eq!(config.get_assigned_hostname(8080), "host-8080");

        let config = create_config(None, None, None, None);
        assert_eq!(config.get_assigned_hostname(8080), "0.0.0.0:8080");
    }

    #[test]
    fn test_get_allow_desired_port() {
        let config = create_config(None, Some(true), None, None);
        assert!(config.get_allow_desired_port());

        let config = create_config(None, None, None, None);
        assert!(config.get_allow_desired_port());
    }
}
