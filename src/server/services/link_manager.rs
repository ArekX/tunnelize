use std::collections::HashMap;

use log::info;
use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use super::{client_manager::ClientInfo, events::ServiceEvent, HandleServiceEvent};

pub struct LinkSession {
    id: Uuid,
    tunnel_id: Uuid,
    client: ClientInfo,
    cancellation_token: CancellationToken,
}

impl From<&LinkSession> for LinkInfo {
    fn from(val: &LinkSession) -> Self {
        LinkInfo {
            id: val.id,
            endpoint_name: val.client.endpoint_name.clone(),
            tunnel_id: val.tunnel_id,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

        Err(format!("Link session not found: {id:?}"))
    }
}

impl HandleServiceEvent for LinkManager {
    async fn handle_event(&mut self, event: &ServiceEvent) {
        match event {
            ServiceEvent::LinkDisconnected { session_id, .. } => {
                self.remove_session(session_id);
            }
            ServiceEvent::TunnelDisconnected { tunnel_id } => {
                for session in self.link_sessions.values() {
                    if &session.tunnel_id == tunnel_id {
                        if let Err(e) = self.cancel_session(&session.id) {
                            info!("Error while cancelling link session: {:?}", e);
                        }
                    }
                }
            }
            ServiceEvent::LinkRejected {
                client_id,
                session_id,
            } => {
                if let Some(session) = self.link_sessions.get(session_id) {
                    if &session.client.id == client_id {
                        self.remove_session(session_id);
                    }
                }
            }
            _ => {}
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_util::sync::CancellationToken;

    fn create_test_client_info() -> ClientInfo {
        ClientInfo {
            id: Uuid::new_v4(),
            endpoint_name: "test_endpoint".to_string(),
        }
    }

    fn create_test_link_manager() -> LinkManager {
        LinkManager::new()
    }

    #[test]
    fn test_create_link_session() {
        let mut manager = create_test_link_manager();
        let client = create_test_client_info();
        let tunnel_id = Uuid::new_v4();
        let cancellation_token = CancellationToken::new();

        let session_id =
            manager.create_link_session(tunnel_id, client.clone(), cancellation_token.clone());

        assert!(manager.link_sessions.contains_key(&session_id));
        let session = manager.link_sessions.get(&session_id).unwrap();
        assert_eq!(session.client.id, client.id);
        assert_eq!(session.tunnel_id, tunnel_id);
    }

    #[test]
    fn test_resolve_tunnel_session_client() {
        let mut manager = create_test_link_manager();
        let client = create_test_client_info();
        let tunnel_id = Uuid::new_v4();
        let cancellation_token = CancellationToken::new();

        let session_id =
            manager.create_link_session(tunnel_id, client.clone(), cancellation_token.clone());

        let result = manager.resolve_tunnel_session_client(&session_id, &tunnel_id);
        assert!(result.is_some());
        let (resolved_client_id, _) = result.unwrap();
        assert_eq!(resolved_client_id, client.id);
    }

    #[test]
    fn test_remove_session() {
        let mut manager = create_test_link_manager();
        let client = create_test_client_info();
        let tunnel_id = Uuid::new_v4();
        let cancellation_token = CancellationToken::new();

        let session_id = manager.create_link_session(tunnel_id, client, cancellation_token);
        manager.remove_session(&session_id);

        assert!(!manager.link_sessions.contains_key(&session_id));
    }

    #[test]
    fn test_get_count() {
        let mut manager = create_test_link_manager();
        assert_eq!(manager.get_count(), 0);

        let client = create_test_client_info();
        let tunnel_id = Uuid::new_v4();
        let cancellation_token = CancellationToken::new();

        manager.create_link_session(tunnel_id, client, cancellation_token);
        assert_eq!(manager.get_count(), 1);
    }

    #[test]
    fn test_list_all_sessions() {
        let mut manager = create_test_link_manager();
        let client = create_test_client_info();
        let tunnel_id = Uuid::new_v4();
        let cancellation_token = CancellationToken::new();

        let session_id = manager.create_link_session(tunnel_id, client.clone(), cancellation_token);
        let sessions = manager.list_all_sessions();

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].id, session_id);
        assert_eq!(sessions[0].endpoint_name, client.endpoint_name);
        assert_eq!(sessions[0].tunnel_id, tunnel_id);
    }

    #[test]
    fn test_get_session_info() {
        let mut manager = create_test_link_manager();
        let client = create_test_client_info();
        let tunnel_id = Uuid::new_v4();
        let cancellation_token = CancellationToken::new();

        let session_id = manager.create_link_session(tunnel_id, client.clone(), cancellation_token);
        let session_info = manager.get_session_info(&session_id);

        assert!(session_info.is_some());
        let session_info = session_info.unwrap();
        assert_eq!(session_info.id, session_id);
        assert_eq!(session_info.endpoint_name, client.endpoint_name);
        assert_eq!(session_info.tunnel_id, tunnel_id);
    }

    #[test]
    fn test_cancel_session() {
        let mut manager = create_test_link_manager();
        let client = create_test_client_info();
        let tunnel_id = Uuid::new_v4();
        let cancellation_token = CancellationToken::new();

        let session_id = manager.create_link_session(tunnel_id, client, cancellation_token.clone());
        assert!(manager.cancel_session(&session_id).is_ok());
        assert!(cancellation_token.is_cancelled());
    }
}
