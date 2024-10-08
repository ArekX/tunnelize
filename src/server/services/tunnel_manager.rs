use std::collections::HashMap;

use uuid::Uuid;

use crate::server::session::tunnel::TunnelSession;

pub struct TunnelManager {
    tunnels: HashMap<Uuid, TunnelSession>,
}

impl TunnelManager {
    pub fn new() -> Self {
        Self {
            tunnels: HashMap::new(),
        }
    }

    pub fn register_tunnel_session(&mut self, tunnel: TunnelSession) {
        self.tunnels.insert(tunnel.get_id(), tunnel);
    }
}
