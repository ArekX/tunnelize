use std::{collections::HashMap, net::SocketAddr};

use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub struct ClientHost {
    client_map: HashMap<Uuid, Client>,
}

pub struct Client {
    pub sender_tx: Sender<Vec<u8>>,
    pub cancel_token: CancellationToken,
    pub address: SocketAddr,
}

impl ClientHost {
    pub fn new() -> Self {
        Self {
            client_map: HashMap::new(),
        }
    }

    pub fn get(&self, client_id: &Uuid) -> Option<&Client> {
        self.client_map.get(client_id)
    }

    pub fn add(
        &mut self,
        sender_tx: Sender<Vec<u8>>,
        cancel_token: CancellationToken,
        address: SocketAddr,
    ) -> Uuid {
        let client_id = Uuid::new_v4();

        self.client_map.insert(
            client_id,
            Client {
                sender_tx,
                cancel_token,
                address,
            },
        );

        client_id
    }

    pub fn remove(&mut self, client_id: &Uuid) {
        self.client_map.remove(client_id);
    }
}
