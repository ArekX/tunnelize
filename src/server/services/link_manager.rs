use std::collections::HashMap;

use uuid::Uuid;

pub struct LinkSession {
    id: Uuid,
    tunnel_id: Uuid,
    endpoint_name: String,
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
    
}
