use std::fmt::Display;

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

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub encryption: Option<EndpointServerEncryption>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub address: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub max_client_input_wait_secs: Option<u64>,

    pub hostname_template: String,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub full_url_template: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub allow_custom_hostnames: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub require_authorization: Option<AuthorizeUser>,
}

impl HttpEndpointConfig {
    pub fn get_address(&self) -> String {
        self.address.clone().unwrap_or_else(|| "0.0.0.0".to_string())
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

        format!("{protocol}://{hostname}{port}")
    }

    pub fn get_is_secure(&self) -> bool {
        match &self.encryption {
            Some(EndpointServerEncryption::None) => false,
            None => false,
            _ => true,
        }
    }

    pub fn get_encryption(&self) -> EndpointServerEncryption {
        self.encryption
            .clone()
            .unwrap_or(EndpointServerEncryption::None)
    }

    pub fn get_max_client_input_wait_secs(&self) -> u64 {
        self.max_client_input_wait_secs
            .unwrap_or(300)
    }

    pub fn get_allow_custom_hostnames(&self) -> bool {
        self.allow_custom_hostnames.unwrap_or(true)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthorizeUser {
    pub realm: Option<String>,
    pub username: String,
    pub password: String,
}

impl Display for AuthorizeUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "\tUsername: {}", self.username)?;
        writeln!(f, "\tPassword: {}", self.password)?;

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HttpPublicEndpointConfig {
    pub port: u16,
    pub is_secure: bool,
    pub address: String,
    pub allow_custom_hostnames: bool,
    pub hostname_template: String,
    pub full_url_template: Option<String>,
    pub require_authorization: Option<AuthorizeUser>,
}

impl From<&HttpEndpointConfig> for HttpPublicEndpointConfig {
    fn from(config: &HttpEndpointConfig) -> Self {
        Self {
            port: config.port,
            is_secure: config.get_is_secure(),
            address: config.get_address(),
            allow_custom_hostnames: config.get_allow_custom_hostnames(),
            hostname_template: config.hostname_template.clone(),
            full_url_template: config.full_url_template.clone(),
            require_authorization: config.require_authorization.clone(),
        }
    }
}

impl Display for HttpPublicEndpointConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Port: {}", self.port)?;
        writeln!(
            f,
            "HTTPS: {}",
            if self.is_secure {
                "Enabled"
            } else {
                "Disabled"
            }
        )?;
        writeln!(f, "Address: {}", self.address)?;
        writeln!(
            f,
            "User can set {{name}} in template: {}",
            if self.allow_custom_hostnames {
                "Allowed"
            } else {
                "Not Allowed"
            }
        )?;
        writeln!(f, "Template: {}", self.hostname_template)?;

        if let Some(authorization) = &self.require_authorization {
            writeln!(f, "Requires clients to authorize:")?;
            write!(f, "{authorization}")?;
        }

        Ok(())
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

        result.validate_child("encryption", &self.get_encryption());

        result.validate_rule_for::<_, MustBeGreaterThanZero>(
            "max_client_input_wait_secs",
            &self.get_max_client_input_wait_secs(),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn get_config() -> HttpEndpointConfig {
        HttpEndpointConfig {
            port: 8080,
            encryption: None,
            address: None,
            max_client_input_wait_secs: None,
            hostname_template: "example.com".to_string(),
            full_url_template: None,
            allow_custom_hostnames: None,
            require_authorization: None,
        }
    }

    #[test]
    fn test_get_address() {
        let mut config = get_config();
        config.address = Some("127.0.0.1".to_string());
        assert_eq!(config.get_address(), "127.0.0.1");

        config.address = None;
        assert_eq!(config.get_address(), "0.0.0.0");
    }

    #[test]
    fn test_get_full_url() {
        let mut config = get_config();
        config.full_url_template = Some("http://{hostname}:{port}".to_string());
        assert_eq!(
            config.get_full_url("example.com"),
            "http://example.com:8080"
        );

        config.full_url_template = None;
        assert_eq!(
            config.get_full_url("example.com"),
            "http://example.com:8080"
        );
    }

    #[test]
    fn test_get_is_secure() {
        let mut config = get_config();
        config.encryption = Some(EndpointServerEncryption::None);
        assert!(!config.get_is_secure());

        config.encryption = Some(EndpointServerEncryption::Tls {
            cert_path: None,
            key_path: None,
        });
        assert!(config.get_is_secure());
    }

    #[test]
    fn test_get_max_client_input_wait_secs() {
        let mut config = get_config();
        config.max_client_input_wait_secs = Some(100);
        assert_eq!(config.get_max_client_input_wait_secs(), 100);

        config.max_client_input_wait_secs = None;
        assert_eq!(config.get_max_client_input_wait_secs(), 300);
    }

    #[test]
    fn test_get_allow_custom_hostnames() {
        let mut config = get_config();
        config.allow_custom_hostnames = Some(false);
        assert!(!config.get_allow_custom_hostnames());

        config.allow_custom_hostnames = None;
        assert!(config.get_allow_custom_hostnames());
    }
}
