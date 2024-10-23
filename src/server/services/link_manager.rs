use std::collections::HashMap;

use log::info;
use serde::Serialize;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use super::{client_manager::ClientInfo, events::ServiceEvent, HandleServiceEvent};

pub struct LinkSession {
    id: Uuid,
    tunnel_id: Uuid,
    client: ClientInfo,
    cancellation_token: CancellationToken,
}

impl Into<LinkInfo> for &LinkSession {
    fn into(self) -> LinkInfo {
        LinkInfo {
            id: self.id,
            endpoint_name: self.client.endpoint_name.clone(),
            tunnel_id: self.tunnel_id,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct LinkInfo {
    pub id: Uuid,
    pub endpoint_name: String,
    pub tunnel_id: Uuid,
}

pub struct LinkManager {
    link_sessions: HashMap<Uuid, LinkSession>,
}

impl LinkManager {
    pub fn new() -> Self {
        Self {
            link_sessions: HashMap::new(),
        }
    }

    pub fn create_link_session(
        &mut self,
        tunnel_id: Uuid,
        client: ClientInfo,
        cancellation_token: CancellationToken,
    ) -> Uuid {
        let id = Uuid::new_v4();
        self.link_sessions.insert(
            id,
            LinkSession {
                id,
                tunnel_id,
                client,
                cancellation_token,
            },
        );
        id
    }

    pub fn resolve_tunnel_session_client(
        &mut self,
        session_id: &Uuid,
        tunnel_id: &Uuid,
    ) -> Option<(Uuid, CancellationToken)> {
        let Some(session) = self.link_sessions.get(session_id) else {
            println!("Session not found");
            return None;
        };

        if &session.tunnel_id != tunnel_id {
            println!("Tunnel ID mismatch");
            return None;
        }

        Some((session.client.id, session.cancellation_token.clone()))
    }

    pub fn remove_session(&mut self, id: &Uuid) {
        info!("Removing link session: {:?}", id);
        self.link_sessions.remove(id);
    }

    pub fn get_count(&self) -> usize {
        self.link_sessions.len()
    }

    pub fn list_all_sessions(&self) -> Vec<LinkInfo> {
        self.link_sessions
            .values()
            .map(|session| session.into())
            .collect()
    }

    pub fn get_session_info(&self, id: &Uuid) -> Option<LinkInfo> {
        self.link_sessions.get(id).map(|session| session.into())
    }

    pub fn cancel_session(&self, id: &Uuid) -> Result<(), String> {
        if let Some(session) = self.link_sessions.get(id) {
            session.cancellation_token.cancel();
            return Ok(());
        }

        Err(format!("Link session not found: {:?}", id))
    }
}

impl HandleServiceEvent for LinkManager {
    async fn handle_event(&mut self, event: &ServiceEvent) {
        match event {
            ServiceEvent::LinkDisconnected { session_id, .. } => {
                self.remove_session(session_id);
            }
            _ => {}
        };
    }
}
