use serde::{Deserialize, Serialize};

// TODO: Add max proxies per tunnel
// TODO: Add max tunnels

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HttpEndpointConfig {
    pub port: u16,
    pub is_secure: bool,
    pub address: Option<String>,
    pub max_client_input_wait_secs: u64,
    pub hostname_template: String,
    pub full_url_template: Option<String>,
    pub allow_custom_hostnames: bool,
    pub require_authorization: Option<AuthorizeUser>,
}

impl HttpEndpointConfig {
    pub fn get_bind_address(&self) -> String {
        let address = self.address.clone().unwrap_or_else(|| format!("0.0.0.0"));

        format!("{}:{}", address, self.port)
    }

    pub fn get_full_url(&self, hostname: &str) -> String {
        if let Some(template) = &self.full_url_template {
            return template
                .replace("{hostname}", hostname)
                .replace("{port}", &self.port.to_string());
        }

        let protocol = if self.is_secure { "https" } else { "http" };
        let port = if self.is_secure && self.port == 443 || !self.is_secure && self.port == 80 {
            "".to_owned()
        } else {
            format!(":{}", self.port)
        };

        format!("{}://{}{}", protocol, hostname, port)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthorizeUser {
    pub realm: Option<String>,
    pub username: String,
    pub password: String,
}
