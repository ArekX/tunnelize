use std::collections::HashMap;

use uuid::Uuid;

pub struct TunnelHost {
    host_tunnel_map: HashMap<String, Uuid>,
}

impl TunnelHost {
    pub fn new() -> Self {
        Self {
            host_tunnel_map: HashMap::new(),
        }
    }

    pub fn add_tunnel(&mut self, hostname: String, tunnel_id: Uuid) {
        self.host_tunnel_map.insert(hostname, tunnel_id);
    }

    pub fn remove_tunnel_by_id(&mut self, tunnel_id: &Uuid) {
        self.host_tunnel_map.retain(|_, v| v != tunnel_id);
    }

    pub fn get_tunnel_id(&self, host: &str) -> Option<&Uuid> {
        self.host_tunnel_map.get(host)
    }
}
