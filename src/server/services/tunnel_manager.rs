use std::collections::HashMap;

use log::debug;
use uuid::Uuid;

use crate::server::session::tunnel::TunnelSession;

use super::{events::ServiceEvent, HandleServiceEvent};

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

    pub fn remove_tunnel_session(&mut self, id: Uuid) {
        self.tunnels.remove(&id);
    }
}

impl HandleServiceEvent for TunnelManager {
    fn handle_event(&mut self, event: ServiceEvent) {
        match event {
            ServiceEvent::TunnelConnected {
                tunnel_session: tunnel,
            } => {
                debug!("Registering tunnel ID to manager: {:?}", tunnel.get_id());
                self.register_tunnel_session(tunnel);
            }
            ServiceEvent::TunnelDisconnected { tunnel_id } => {
                debug!("Removing tunnel ID from manager: {:?}", tunnel_id);
                self.remove_tunnel_session(tunnel_id);
            }
        }
    }
}
