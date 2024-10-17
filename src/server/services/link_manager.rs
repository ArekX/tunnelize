use std::collections::HashMap;

use uuid::Uuid;

use crate::common::connection::ConnectionStream;

pub struct LinkSession {
    id: Uuid,
    tunnel_id: Uuid,
    client_id: Uuid,
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

    pub fn create_link_session(&mut self, tunnel_id: Uuid, client_id: Uuid) -> Uuid {
        let id = Uuid::new_v4();
        self.link_sessions.insert(
            id,
            LinkSession {
                id,
                tunnel_id,
                client_id,
            },
        );
        id
    }

    pub fn resolve_tunnel_session_client(
        &mut self,
        session_id: &Uuid,
        tunnel_id: &Uuid,
    ) -> Option<Uuid> {
        let Some(session) = self.link_sessions.get(session_id) else {
            return None;
        };

        if &session.tunnel_id != tunnel_id {
            return None;
        }

        Some(session.client_id)
    }

    pub fn remove_session(&mut self, id: &Uuid) {
        self.link_sessions.remove(id);
    }
}
