use std::collections::HashMap;

use log::info;
use uuid::Uuid;

use crate::common::connection::ConnectionStream;

use super::{events::ServiceEvent, HandleServiceEvent};

pub struct ClientLink {
    pub stream: ConnectionStream,
    pub initial_tunnel_data: Option<Vec<u8>>,
}

pub struct Client {
    id: Uuid,
    service_name: String,
    hostname: String,
    link: Option<ClientLink>,
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
            link: Some(ClientLink {
                stream,
                initial_tunnel_data,
            }),
        }
    }

    pub fn get_id(&self) -> Uuid {
        self.id
    }
}

pub struct ClientManager {
    clients: HashMap<Uuid, Client>,
}

impl ClientManager {
    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
        }
    }

    pub fn subscribe_client(&mut self, client: Client) {
        self.clients.insert(client.id, client);
    }

    pub async fn cancel_client(&mut self, id: &Uuid, cancel_with_data: &Vec<u8>) {
        if let Some(mut link) = self.take_client_link(id) {
            link.stream.close_with_data(cancel_with_data).await;
        }

        self.remove_client(id);
    }

    pub fn take_client_link(&mut self, id: &Uuid) -> Option<ClientLink> {
        self.clients
            .get_mut(id)
            .and_then(|client| client.link.take())
    }

    pub fn remove_client(&mut self, id: &Uuid) {
        info!("Client disconnected: {:?}", id);
        self.clients.remove(id);
    }
}

impl HandleServiceEvent for ClientManager {
    async fn handle_event(&mut self, event: &ServiceEvent) {
        match event {
            ServiceEvent::LinkDisconnected { client_id, .. } => {
                self.remove_client(client_id);
            }
            _ => {}
        };
    }
}
