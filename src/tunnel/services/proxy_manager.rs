use std::collections::HashMap;

use uuid::Uuid;

use crate::{
    server::endpoints::messages::EndpointInfo,
    tunnel::configuration::{TunnelConfiguration, TunnelProxy},
};

pub struct ProxySession {
    pub proxy_id: Uuid,
    pub endpoint_name: String,
    pub forward_address: String,
    pub endpoint_info: Option<EndpointInfo>,
}

pub struct ProxyManager {
    proxy_session_map: HashMap<Uuid, ProxySession>,
}

impl ProxyManager {
    pub fn new(config: &TunnelConfiguration) -> Self {
        let mut instance = Self {
            proxy_session_map: HashMap::new(),
        };

        for proxy in &config.proxies {
            instance.add_proxy(proxy);
        }

        instance
    }

    pub fn add_proxy(&mut self, proxy: &TunnelProxy) -> Uuid {
        let id = Uuid::new_v4();

        let proxy_session = ProxySession {
            proxy_id: id,
            endpoint_name: proxy.endpoint_name.clone(),
            forward_address: proxy.forward_address.clone(),
            endpoint_info: None,
        };

        self.proxy_session_map.insert(id, proxy_session);

        id
    }

    pub fn get_proxy_list(&self) -> Vec<Proxy> {
        self.proxy_session_map.values().map(|v| {

        })
    }
}

