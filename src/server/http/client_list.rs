use std::collections::HashMap;

use tokio::net::TcpStream;
use uuid::Uuid;

pub struct ClientList {
    client_map: HashMap<Uuid, Client>,
}

pub struct Client {
    pub initial_request: String,
    pub stream: TcpStream,
}

impl ClientList {
    pub fn new() -> Self {
        ClientList {
            client_map: HashMap::new(),
        }
    }

    pub fn issue_client_id(&mut self) -> Uuid {
        Uuid::new_v4()
    }

    pub fn register(&mut self, client_id: Uuid, stream: TcpStream, initial_request: String) {
        self.client_map.insert(
            client_id,
            Client {
                initial_request,
                stream,
            },
        );
    }

    pub fn is_registered(&self, id: Uuid) -> bool {
        self.client_map.contains_key(&id)
    }

    pub fn release(&mut self, id: Uuid) -> Option<Client> {
        if let Some((_, client)) = self.client_map.remove_entry(&id) {
            return Some(client);
        }

        None
    }
}
