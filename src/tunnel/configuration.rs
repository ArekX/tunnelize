use serde::{Deserialize, Serialize};

use super::proxies::http::HttpProxy;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TunnelConfiguration {
    pub server_host: String,
    pub endpoint_key: Option<String>,
    pub admin_key: Option<String>,
    pub proxies: Vec<TunnelProxy>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TunnelProxy {
    pub service_name: String,
    pub proxy: ProxyConfiguration,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProxyConfiguration {
    Http(HttpProxy),
}
