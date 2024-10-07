use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TunnelConfiguration {
    pub server_host: String,
    pub endpoint_key: Option<String>,
    pub admin_key: Option<String>,
    pub proxies: Vec<ProxyConfiguration>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProxyConfiguration {}
