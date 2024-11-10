use serde::{Deserialize, Serialize};

use crate::{
    common::{
        validate::{Validatable, Validation},
        validate_rules::{
            HostAddressMustBeValid, HostnameTemplatemustBeValid, MustBeGreaterThanZero,
            MustNotBeEmptyString, PortMustBeValid,
        },
    },
    server::configuration::EndpointServerEncryption,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HttpEndpointConfig {
    pub port: u16,
    pub encryption: EndpointServerEncryption,
    pub address: Option<String>,
    pub max_client_input_wait_secs: u64,
    pub hostname_template: String,
    pub full_url_template: Option<String>,
    pub allow_custom_hostnames: bool,
    pub require_authorization: Option<AuthorizeUser>,
}

impl HttpEndpointConfig {
    pub fn get_address(&self) -> String {
        self.address.clone().unwrap_or_else(|| format!("0.0.0.0"))
    }

    pub fn get_full_url(&self, hostname: &str) -> String {
        if let Some(template) = &self.full_url_template {
            return template
                .replace("{hostname}", hostname)
                .replace("{port}", &self.port.to_string());
        }

        let protocol = if self.get_is_secure() {
            "https"
        } else {
            "http"
        };
        let port = if self.get_is_secure() && self.port == 443
            || !self.get_is_secure() && self.port == 80
        {
            "".to_owned()
        } else {
            format!(":{}", self.port)
        };

        format!("{}://{}{}", protocol, hostname, port)
    }

    pub fn get_is_secure(&self) -> bool {
        match &self.encryption {
            EndpointServerEncryption::None => false,
            _ => true,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthorizeUser {
    pub realm: Option<String>,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HttpPublicEndpointConfig {
    pub port: u16,
    pub is_secure: bool,
    pub address: Option<String>,
    pub allow_custom_hostnames: bool,
    pub hostname_template: String,
}

impl From<&HttpEndpointConfig> for HttpPublicEndpointConfig {
    fn from(config: &HttpEndpointConfig) -> Self {
        Self {
            port: config.port,
            is_secure: config.get_is_secure(),
            address: config.address.clone(),
            allow_custom_hostnames: config.allow_custom_hostnames,
            hostname_template: config.hostname_template.clone(),
        }
    }
}

impl Validatable for AuthorizeUser {
    fn validate(&self, result: &mut Validation) {
        result.validate_rule::<MustNotBeEmptyString>("username", &self.username);
        result.validate_rule::<MustNotBeEmptyString>("password", &self.password);

        if let Some(realm) = &self.realm {
            result.validate_rule::<MustNotBeEmptyString>("realm", realm);
        }
    }
}

impl Validatable for HttpEndpointConfig {
    fn validate(&self, result: &mut Validation) {
        if let Some(address) = &self.address {
            result.validate_rule::<HostAddressMustBeValid>("address", address);
        }
        result.validate_rule::<PortMustBeValid>("port", &self.port);

        result.validate_child("encryption", &self.encryption);

        result.validate_rule_for::<_, MustBeGreaterThanZero>(
            "max_client_input_wait_secs",
            &self.max_client_input_wait_secs,
        );

        result.validate_rule::<HostnameTemplatemustBeValid>(
            "hostname_template",
            &self.hostname_template,
        );

        if let Some(authorization) = &self.require_authorization {
            result.validate_child("authorization", authorization);
        }
    }
}
