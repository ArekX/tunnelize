use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TunnelConfiguration {
    pub server_host: String,
    pub endpoint_key: Option<String>,
    pub admin_key: Option<String>,
    pub proxies: Vec<TunnelProxy>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TunnelProxy {
    pub endpoint_name: String,
    pub forward_address: String,
    pub config: ProxyConfiguration,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ProxyConfiguration {
    Http { desired_name: Option<String> },
    Tcp { port_from: u16, port_to: u16 },
    Udp { port_from: u16, port_to: u16 },
}
