use std::{collections::HashMap, net::SocketAddr, time::Instant};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub struct Host {
    id: Uuid,
    address: SocketAddr,
    cancel_token: CancellationToken,
    last_activity: Instant,
    tunnel_id: Uuid,
}

impl Host {
    pub fn new(
        id: Uuid,
        address: SocketAddr,
        tunnel_id: Uuid,
        cancel_token: CancellationToken,
    ) -> Self {
        Self {
            id,
            address,
            cancel_token,
            last_activity: Instant::now(),
            tunnel_id,
        }
    }

    pub fn is_inactive(&self, timeout: u64) -> bool {
        self.cancel_token.is_cancelled() || self.last_activity.elapsed().as_secs() > timeout
    }

    pub fn cancel(&self) {
        self.cancel_token.cancel();
    }

    pub fn update_activity(&mut self) {
        self.last_activity = Instant::now();
    }
}

pub struct ClientHost {
    address_client_map: HashMap<SocketAddr, Uuid>,
    client_map: HashMap<Uuid, Host>,
}

impl ClientHost {
    pub fn new() -> Self {
        Self {
            address_client_map: HashMap::new(),
            client_map: HashMap::new(),
        }
    }

    pub fn add(&mut self, client: Host) {
        self.client_map.insert(client.id, client);
    }

    pub fn get_client_address(&self, client_id: &Uuid) -> Option<SocketAddr> {
        self.client_map
            .get(client_id)
            .map(|client| client.address.clone())
    }

    pub async fn cleanup_by_tunnel(&mut self, tunnel_id: &Uuid) {
        self.client_map.retain(|_, client| {
            if &client.tunnel_id == tunnel_id {
                client.cancel();
                self.address_client_map.remove(&client.address);
                false
            } else {
                true
            }
        });
    }

    pub async fn cleanup_inactive_clients(&mut self, timeout: u64) -> Vec<Uuid> {
        let mut inactive_clients = Vec::new();

        self.client_map.retain(|_, client| {
            if client.is_inactive(timeout) {
                client.cancel();
                self.address_client_map.remove(&client.address);
                inactive_clients.push(client.id);
                false
            } else {
                true
            }
        });

        inactive_clients
    }

    pub fn cancel_client(&mut self, client_id: &Uuid) {
        if let Some(client) = self.client_map.get(client_id) {
            client.cancel();
            self.address_client_map.remove(&client.address);
        }
    }

    pub fn cancel_all(&mut self) {
        for client in self.client_map.values() {
            client.cancel();
        }

        self.client_map.clear();
        self.address_client_map.clear();
    }
}
