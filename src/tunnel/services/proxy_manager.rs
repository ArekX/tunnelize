use std::collections::HashMap;

use uuid::Uuid;

use crate::tunnel::configuration::TunnelProxy;

pub struct ProxySession {
    pub proxy_id: Uuid,
    pub endpoint_name: String,
    pub forward_address: String,
}

pub struct ProxyManager {
    proxy_session_map: HashMap<Uuid, ProxySession>,
}

impl ProxyManager {
    pub fn new() -> Self {
        Self {
            proxy_session_map: HashMap::new(),
        }
    }

    pub fn add_proxy(&mut self, proxy: &TunnelProxy) -> Uuid {
        let id = Uuid::new_v4();

        let proxy_session = ProxySession {
            proxy_id: id,
            endpoint_name: proxy.endpoint_name.clone(),
            forward_address: proxy.forward_address.clone(),
        };

        self.proxy_session_map.insert(id, proxy_session);

        id
    }
}
