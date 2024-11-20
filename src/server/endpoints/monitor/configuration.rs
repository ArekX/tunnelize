use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{
    common::{
        validate::{Validatable, Validation},
        validate_rules::{HostAddressMustBeValid, MustNotBeEmptyString, PortMustBeValid},
    },
    server::configuration::EndpointServerEncryption,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MonitorEndpointConfig {
    pub port: u16,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub encryption: Option<EndpointServerEncryption>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub address: Option<String>,

    pub authentication: MonitorAuthentication,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub allow_cors_origins: Option<MonitorOrigin>,
}

impl MonitorEndpointConfig {
    pub fn get_address(&self) -> String {
        self.address.clone().unwrap_or_else(|| format!("0.0.0.0"))
    }

    pub fn get_encryption(&self) -> EndpointServerEncryption {
        self.encryption
            .clone()
            .unwrap_or_else(|| EndpointServerEncryption::None)
    }

    pub fn get_allow_cors_origins(&self) -> MonitorOrigin {
        self.allow_cors_origins
            .clone()
            .unwrap_or_else(|| MonitorOrigin::Any)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PublicMonitorEndpointConfig {
    pub port: u16,
    pub address: String,
    pub encryption: EndpointServerEncryption,
    pub allow_cors_origins: MonitorOrigin,
}

impl From<&MonitorEndpointConfig> for PublicMonitorEndpointConfig {
    fn from(config: &MonitorEndpointConfig) -> Self {
        PublicMonitorEndpointConfig {
            port: config.port,
            address: config.get_address(),
            encryption: config.get_encryption(),
            allow_cors_origins: config.get_allow_cors_origins(),
        }
    }
}

impl Display for PublicMonitorEndpointConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Port: {}", self.port)?;
        writeln!(
            f,
            "HTTPS: {}",
            if let EndpointServerEncryption::Tls { .. } = self.encryption {
                "Enabled"
            } else {
                "Disabled"
            }
        )?;
        writeln!(f, "Address: {}", self.address)?;
        writeln!(f, "CORS: {}", self.allow_cors_origins)?;

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum MonitorOrigin {
    Any,
    List(Vec<String>),
    None,
}

impl Display for MonitorOrigin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MonitorOrigin::Any => writeln!(f, "Any origin allowed"),
            MonitorOrigin::List(origins) => {
                writeln!(f, "List:")?;

                for origin in origins {
                    writeln!(f, "\t{}", origin)?;
                }

                Ok(())
            }
            MonitorOrigin::None => writeln!(f, "No origin allowed"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "kebab-case")]
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

impl Validatable for MonitorOrigin {
    fn validate(&self, result: &mut Validation) {
        match self {
            MonitorOrigin::List(origins) => {
                for (index, origin) in origins.iter().enumerate() {
                    result.validate_rule::<MustNotBeEmptyString>(
                        &format!("origins.{}", index),
                        origin,
                    );
                }
            }
            _ => {}
        }
    }
}

impl Validatable for MonitorAuthentication {
    fn validate(&self, result: &mut Validation) {
        match self {
            MonitorAuthentication::Basic { username, password } => {
                result.validate_rule::<MustNotBeEmptyString>("username", username);
                result.validate_rule::<MustNotBeEmptyString>("password", password);
            }
            MonitorAuthentication::Bearer { token } => {
                result.validate_rule::<MustNotBeEmptyString>("token", token);
            }
        }
    }
}

impl Validatable for MonitorEndpointConfig {
    fn validate(&self, result: &mut Validation) {
        result.validate_rule::<PortMustBeValid>("port", &self.port);
        result.validate_child("encryption", &self.get_encryption());

        if let Some(address) = &self.address {
            result.validate_rule::<HostAddressMustBeValid>("address", address);
        }
    }
}
