use std::collections::HashMap;

use uuid::Uuid;

use crate::common::connection::ConnectionStream;

pub struct Client {
    id: Uuid,
    service_name: String,
    hostname: String,
    pub stream: ConnectionStream,
    initial_tunnel_data: Option<Vec<u8>>,
}

impl Client {
    pub fn new(
        id: Uuid,
        service_name: String,
        hostname: String,
        stream: ConnectionStream,
        initial_tunnel_data: Option<Vec<u8>>,
    ) -> Self {
        Self {
            id,
            service_name,
            hostname,
            stream,
            initial_tunnel_data,
        }
    }

    pub fn get_id(&self) -> Uuid {
        self.id
    }
}

pub struct ClientManager {
    clients: HashMap<Uuid, Client>,
    // TODO: Separate client streams into separate hashmap
}

impl ClientManager {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    pub fn add_client(&mut self, client: Client) {
        self.clients.insert(client.id, client);
    }

    pub fn take_client(&mut self, id: Uuid) -> Option<Client> {
        self.clients.remove(&id)
    }
}
