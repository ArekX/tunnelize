use std::{
    collections::HashMap,
    io::{Error, ErrorKind},
};

use crate::common::connection::{Connection, ConnectionStreamContext};
use log::info;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{events::ServiceEvent, HandleServiceEvent};

pub struct ClientLink {
    pub stream: Connection,
    pub context: Option<ConnectionStreamContext>,
    pub initial_tunnel_data: Option<Vec<u8>>,
}

pub struct Client {
    id: Uuid,
    endpoint_name: String,
    link: Option<ClientLink>,
}

impl Into<ClientInfo> for &Client {
    fn into(self) -> ClientInfo {
        ClientInfo {
            id: self.id,
            endpoint_name: self.endpoint_name.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClientInfo {
    pub id: Uuid,
    pub endpoint_name: String,
}

impl Client {
    pub fn new(
        id: Uuid,
        endpoint_name: String,
        stream: Connection,
        context: Option<ConnectionStreamContext>,
        initial_tunnel_data: Option<Vec<u8>>,
    ) -> Self {
        Self {
            id,
            endpoint_name,
            link: Some(ClientLink {
                stream,
                context,
                initial_tunnel_data,
            }),
        }
    }

    pub fn get_id(&self) -> Uuid {
        self.id
    }

    pub fn take_link(&mut self) -> Option<ClientLink> {
        self.link.take()
    }
}

pub struct ClientManager {
    clients: HashMap<Uuid, Client>,
    max_clients: usize,
}

impl ClientManager {
    pub fn new(max_clients: usize) -> Self {
        Self {
            clients: HashMap::new(),
            max_clients,
        }
    }

    pub fn subscribe_client(
        &mut self,
        mut client: Client,
    ) -> Result<(), (Error, Option<ClientLink>)> {
        if self.clients.len() >= self.max_clients {
            return Err((
                Error::new(ErrorKind::Other, "Maximum number of clients reached"),
                client.take_link(),
            ));
        }

        self.clients.insert(client.id, client);

        Ok(())
    }

    pub async fn cancel_client(&mut self, id: &Uuid, cancel_with_data: &Option<Vec<u8>>) {
        if let Some(mut link) = self.take_client_link(id) {
            if let Some(cancel_data) = cancel_with_data {
                link.stream.close_with_data(cancel_data).await;
            } else {
                link.stream.shutdown().await;
            }
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

    pub fn get_info(&self, id: &Uuid) -> Option<ClientInfo> {
        self.clients.get(id).map(|client| client.into())
    }

    pub fn list_all_clients(&self) -> Vec<ClientInfo> {
        self.clients.values().map(|client| client.into()).collect()
    }

    pub fn get_count(&self) -> usize {
        self.clients.len()
    }
}

impl HandleServiceEvent for ClientManager {
    async fn handle_event(&mut self, event: &ServiceEvent) {
        match event {
            ServiceEvent::LinkDisconnected { client_id, .. } => {
                self.remove_client(client_id);
            }
            ServiceEvent::LinkRejected { client_id, .. } => {
                self.remove_client(client_id);
            }
            _ => {}
        };
    }
}
