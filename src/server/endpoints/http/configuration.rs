use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HttpEndpointConfig {
    pub port: u16,
    pub max_client_input_wait: u16,
    pub host_template: String,
    pub tunnel_url_template: String,
    pub allow_custom_hostnames: bool,
    pub require_authorization: Option<AuthorizeUser>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthorizeUser {
    pub realm: Option<String>,
    pub username: String,
    pub password: String,
}
