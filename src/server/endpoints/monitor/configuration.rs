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
        result.validate_child("encryption", &self.encryption);

        if let Some(address) = &self.address {
            result.validate_rule::<HostAddressMustBeValid>("address", address);
        }
    }
}
