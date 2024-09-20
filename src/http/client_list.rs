use std::collections::HashMap;

use tokio::net::TcpStream;

pub struct ClientList {
    id_counter: u32,
    client_map: HashMap<u32, Client>,
}

pub struct Client {
    pub initial_request: String,
    pub stream: TcpStream,
}

impl ClientList {
    pub fn new() -> Self {
        ClientList {
            id_counter: 0,
            client_map: HashMap::new(),
        }
    }

    pub fn issue_client_id(&mut self) -> u32 {
        let issued_id = self.id_counter;
        self.id_counter = self.id_counter.wrapping_add(1);
        issued_id
    }

    pub fn register(&mut self, client_id: u32, stream: TcpStream, initial_request: String) -> u32 {
        self.client_map.insert(
            client_id,
            Client {
                initial_request,
                stream,
            },
        );

        client_id
    }

    pub fn is_registered(&self, id: u32) -> bool {
        self.client_map.contains_key(&id)
    }

    pub fn release(&mut self, id: u32) -> Option<Client> {
        if let Some((_, client)) = self.client_map.remove_entry(&id) {
            return Some(client);
        }

        None
    }
}
