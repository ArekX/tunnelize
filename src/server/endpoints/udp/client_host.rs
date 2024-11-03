use std::{collections::HashMap, net::SocketAddr, time::Instant};

use log::error;
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub struct Client {
    id: Uuid,
    port: u16,
    address: SocketAddr,
    socket_tx: Sender<Vec<u8>>,
    cancel_token: CancellationToken,
    last_activity: Instant,
    connected_tunnel_id: Option<Uuid>,
}

impl Client {
    pub fn new(
        id: Uuid,
        port: u16,
        address: SocketAddr,
        socket_tx: Sender<Vec<u8>>,
        cancel_token: CancellationToken,
    ) -> Self {
        Self {
            id,
            port,
            address,
            socket_tx,
            cancel_token,
            last_activity: Instant::now(),
            connected_tunnel_id: None,
        }
    }

    pub fn is_inactive(&self, timeout: u64) -> bool {
        self.cancel_token.is_cancelled() || self.last_activity.elapsed().as_secs() > timeout
    }

    pub fn cancel(&self) {
        self.cancel_token.cancel();
    }

    pub fn connect_tunnel(&mut self, tunnel_id: Uuid) {
        self.connected_tunnel_id = Some(tunnel_id);
    }

    pub fn update_activity(&mut self) {
        self.last_activity = Instant::now();
    }

    pub async fn send(&mut self, data: Vec<u8>) {
        match self.socket_tx.send(data).await {
            Ok(_) => {
                self.update_activity();
            }
            Err(e) => {
                error!("Failed to send data to client: {}", e);
            }
        }
    }
}

pub struct ClientHost {
    address_client_map: HashMap<SocketAddr, Uuid>,
    client_map: HashMap<Uuid, Client>,
}

impl ClientHost {
    pub fn new() -> Self {
        Self {
            address_client_map: HashMap::new(),
            client_map: HashMap::new(),
        }
    }

    pub fn add(&mut self, client: Client) {
        self.client_map.insert(client.id, client);
    }

    pub fn client_exists(&self, address: &SocketAddr) -> bool {
        self.address_client_map.contains_key(address)
    }

    pub fn get_client_address(&self, client_id: &Uuid) -> Option<&SocketAddr> {
        self.client_map.get(client_id).map(|client| &client.address)
    }

    pub async fn send(&mut self, address: &SocketAddr, data: Vec<u8>) {
        if let Some(client_id) = self.address_client_map.get(address) {
            if let Some(client) = self.client_map.get_mut(client_id) {
                client.send(data).await;
            }
        }
    }

    pub fn update_activity(&mut self, client_id: &Uuid) {
        if let Some(client) = self.client_map.get_mut(client_id) {
            client.update_activity();
        }
    }

    pub async fn cleanup_by_tunnel(&mut self, tunnel_id: &Uuid) {
        self.client_map.retain(|_, client| {
            let Some(connected_tunnel_id) = client.connected_tunnel_id.as_ref() else {
                return true;
            };

            if connected_tunnel_id == tunnel_id {
                client.cancel();
                self.address_client_map.remove(&client.address);
                false
            } else {
                true
            }
        });
    }

    pub async fn cleanup_inactive_clients(&mut self, timeout: u64) {
        self.client_map.retain(|_, client| {
            if client.is_inactive(timeout) {
                client.cancel();
                self.address_client_map.remove(&client.address);
                false
            } else {
                true
            }
        });
    }

    pub fn connect_tunnel(&mut self, port: u16, tunnel_id: Uuid) {
        for client in self.client_map.values_mut() {
            if client.port == port {
                client.connect_tunnel(tunnel_id);
            }
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
