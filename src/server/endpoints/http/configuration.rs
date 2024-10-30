use serde::{Deserialize, Serialize};

use crate::server::configuration::EndpointServerEncryption;

// TODO: Add max proxies per tunnel
// TODO: Add max tunnels

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
}

impl From<&HttpEndpointConfig> for HttpPublicEndpointConfig {
    fn from(config: &HttpEndpointConfig) -> Self {
        Self {
            port: config.port,
            is_secure: config.get_is_secure(),
            address: config.address.clone(),
            allow_custom_hostnames: config.allow_custom_hostnames,
        }
    }
}
