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

    pub fn get_client_id(&mut self, id: &Uuid) -> Option<Uuid> {
        self.link_sessions.get(id).map(|session| session.client_id)
    }

    pub fn get_tunnel_id(&mut self, id: &Uuid) -> Option<Uuid> {
        self.link_sessions.get(id).map(|session| session.tunnel_id)
    }

    pub fn remove_session(&mut self, id: &Uuid) {
        self.link_sessions.remove(id);
    }
}
