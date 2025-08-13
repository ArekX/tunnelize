use std::collections::HashMap;

use log::debug;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    common::channel::{DataResponse, RequestSender},
    server::session::{
        messages::{TunnelChannelRequest, TunnelChannelResponse},
        tunnel::{TunnelProxyInfo, TunnelSession},
    },
};

use super::{events::ServiceEvent, HandleServiceEvent};

pub struct TunnelManager {
    tunnels: HashMap<Uuid, TunnelSession>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TunnelInfo {
    pub id: Uuid,
    pub name: Option<String>,
    pub proxies: Vec<TunnelProxyInfo>,
    pub last_heartbeat_timestamp: i64,
}

impl TunnelManager {
    pub fn new() -> Self {
        Self {
            tunnels: HashMap::new(),
        }
    }

    pub fn get_session_tx(&self, id: &Uuid) -> Option<RequestSender<TunnelChannelRequest>> {
        self.tunnels.get(id).map(|session| session.get_channel_tx())
    }

    pub fn cancel_session(&self, id: &Uuid) -> Result<(), String> {
        if let Some(session) = self.tunnels.get(id) {
            session.cancel();
            return Ok(());
        }

        Err(format!("Tunnel session not found: {id:?}"))
    }

    pub async fn send_session_request<T: Into<TunnelChannelRequest> + DataResponse>(
        &self,
        id: &Uuid,
        request: T,
    ) -> tokio::io::Result<T::Response>
    where
        T::Response: TryFrom<TunnelChannelResponse>,
    {
        let Some(tunnel_tx) = self.get_session_tx(id) else {
            return Err(tokio::io::Error::new(
                tokio::io::ErrorKind::NotFound,
                format!("Tunnel session not found: {id:?}"),
            ));
        };

        tunnel_tx.request(request).await
    }

    pub fn get_count(&self) -> usize {
        self.tunnels.len()
    }

    pub fn register_tunnel_session(&mut self, tunnel: &TunnelSession) {
        self.tunnels.insert(tunnel.get_id(), tunnel.clone());
    }

    pub fn remove_tunnel_session(&mut self, id: &Uuid) {
        self.tunnels.remove(id);
    }

    pub fn list_all_tunnels(&self) -> Vec<TunnelInfo> {
        self.tunnels.values().map(|tunnel| tunnel.into()).collect()
    }

    pub fn get_tunnel_info(&self, id: &Uuid) -> Option<TunnelInfo> {
        self.tunnels.get(id).map(|tunnel| tunnel.into())
    }

    pub fn update_last_heartbeat(&mut self, id: &Uuid) {
        if let Some(tunnel) = self.tunnels.get_mut(id) {
            tunnel.update_heartbeat_timestamp();
        }
    }

    pub fn is_tunnel_stale(&self, id: &Uuid) -> bool {
        if let Some(tunnel) = self.tunnels.get(id) {
            return tunnel.is_stale();
        }
        false
    }
}

impl HandleServiceEvent for TunnelManager {
    async fn handle_event(&mut self, event: &ServiceEvent) {
        match event {
            ServiceEvent::TunnelConnected { tunnel_session } => {
                debug!(
                    "Registering tunnel ID to manager: {:?}",
                    tunnel_session.get_id()
                );
                self.register_tunnel_session(tunnel_session);
            }
            ServiceEvent::TunnelDisconnected { tunnel_id } => {
                debug!("Removing tunnel ID from manager: {:?}", tunnel_id);
                self.remove_tunnel_session(tunnel_id);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::session::tunnel::{create, TunnelSession};
    use uuid::Uuid;

    fn create_tunnel_session(id: Uuid) -> TunnelSession {
        let (session, _) = create(id, None, vec![]);
        session
    }

    #[test]
    fn test_new() {
        let manager = TunnelManager::new();
        assert_eq!(manager.get_count(), 0);
    }

    #[test]
    fn test_register_tunnel_session() {
        let mut manager = TunnelManager::new();
        let id = Uuid::new_v4();
        let session = create_tunnel_session(id);

        manager.register_tunnel_session(&session);
        assert_eq!(manager.get_count(), 1);
        assert!(manager.get_tunnel_info(&id).is_some());
    }

    #[test]
    fn test_remove_tunnel_session() {
        let mut manager = TunnelManager::new();
        let id = Uuid::new_v4();
        let session = create_tunnel_session(id);

        manager.register_tunnel_session(&session);
        manager.remove_tunnel_session(&id);
        assert_eq!(manager.get_count(), 0);
        assert!(manager.get_tunnel_info(&id).is_none());
    }

    #[test]
    fn test_get_session_tx() {
        let mut manager = TunnelManager::new();
        let id = Uuid::new_v4();
        let session = create_tunnel_session(id);

        manager.register_tunnel_session(&session);
        assert!(manager.get_session_tx(&id).is_some());
    }

    #[test]
    fn test_cancel_session() {
        let mut manager = TunnelManager::new();
        let id = Uuid::new_v4();
        let session = create_tunnel_session(id);

        manager.register_tunnel_session(&session);
        assert!(manager.cancel_session(&id).is_ok());
    }

    #[test]
    fn test_list_all_tunnels() {
        let mut manager = TunnelManager::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let session1 = create_tunnel_session(id1);
        let session2 = create_tunnel_session(id2);

        manager.register_tunnel_session(&session1);
        manager.register_tunnel_session(&session2);
        let tunnels = manager.list_all_tunnels();
        assert_eq!(tunnels.len(), 2);
    }

    #[test]
    fn test_get_tunnel_info() {
        let mut manager = TunnelManager::new();
        let id = Uuid::new_v4();
        let session = create_tunnel_session(id);

        manager.register_tunnel_session(&session);
        let info = manager.get_tunnel_info(&id);
        assert!(info.is_some());
    }
}
