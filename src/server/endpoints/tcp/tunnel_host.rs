use std::collections::HashMap;

use uuid::Uuid;

pub struct TunnelHost {
    host_tunnel_map: HashMap<u16, Tunnel>,
}

pub struct Tunnel {
    pub tunnel_id: Uuid,
    pub proxy_id: Uuid,
}

impl TunnelHost {
    pub fn new() -> Self {
        Self {
            host_tunnel_map: HashMap::new(),
        }
    }

    pub fn add_tunnel(&mut self, port: u16, tunnel_id: &Uuid, proxy_id: &Uuid) {
        self.host_tunnel_map.insert(
            port,
            Tunnel {
                tunnel_id: *tunnel_id,
                proxy_id: *proxy_id,
            },
        );
    }

    pub fn remove_tunnel(&mut self, tunnel_id: &Uuid) {
        self.host_tunnel_map
            .retain(|_, v| &v.tunnel_id != tunnel_id);
    }

    pub fn get_tunnel(&self, port: u16) -> Option<&Tunnel> {
        self.host_tunnel_map.get(&port)
    }
}
