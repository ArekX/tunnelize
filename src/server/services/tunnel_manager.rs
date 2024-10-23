use std::collections::HashMap;

use log::debug;
use serde::Serialize;
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

#[derive(Debug, Serialize)]
pub struct TunnelInfo {
    pub id: Uuid,
    pub name: Option<String>,
    pub proxies: Vec<TunnelProxyInfo>,
}

impl TunnelManager {
    pub fn new() -> Self {
        Self {
            tunnels: HashMap::new(),
        }
    }

    pub fn get_session_tx(&self, id: &Uuid) -> Option<RequestSender<TunnelChannelRequest>> {
        match self.tunnels.get(id) {
            Some(session) => Some(session.get_channel_tx()),
            None => None,
        }
    }

    pub fn cancel_session(&self, id: &Uuid) -> Result<(), String> {
        if let Some(session) = self.tunnels.get(id) {
            session.cancel();
            return Ok(());
        }

        Err(format!("Tunnel session not found: {:?}", id))
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
                format!("Tunnel session not found: {:?}", id),
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
        self.tunnels.remove(&id);
    }

    pub fn list_all_tunnels(&self) -> Vec<TunnelInfo> {
        self.tunnels.values().map(|tunnel| tunnel.into()).collect()
    }

    pub fn get_tunnel_info(&self, id: &Uuid) -> Option<TunnelInfo> {
        self.tunnels.get(id).map(|tunnel| tunnel.into())
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
