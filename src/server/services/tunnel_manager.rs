use std::collections::HashMap;

use log::debug;
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

use crate::server::session::{messages::TunnelSessionMessage, tunnel::TunnelSession};

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

    pub fn get_session_tx(&self, id: &Uuid) -> Option<Sender<TunnelSessionMessage>> {
        match self.tunnels.get(id) {
            Some(session) => Some(session.get_channel_tx()),
            None => None,
        }
    }

    pub fn register_tunnel_session(&mut self, tunnel: &TunnelSession) {
        self.tunnels.insert(tunnel.get_id(), tunnel.clone());
    }

    pub fn remove_tunnel_session(&mut self, id: &Uuid) {
        self.tunnels.remove(&id);
    }
}

impl HandleServiceEvent for TunnelManager {
    async fn handle_event(&mut self, event: &ServiceEvent) {
        match event {
            ServiceEvent::TunnelConnected {
                tunnel_session: tunnel,
                tunnel_proxies: _, // TODO: Add proxy configuration to tunnel list
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
