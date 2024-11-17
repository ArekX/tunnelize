use std::{
    collections::HashMap,
    io::{Error, ErrorKind},
};

use crate::common::connection::{Connection, ConnectionStreamContext};
use log::info;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{events::ServiceEvent, HandleServiceEvent};

#[derive(Debug)]
pub struct ClientLink {
    pub stream: Connection,
    pub context: Option<ConnectionStreamContext>,
    pub initial_tunnel_data: Option<Vec<u8>>,
}

#[derive(Debug)]
pub struct Client {
    id: Uuid,
    endpoint_name: String,
    link: Option<ClientLink>,
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

    #[cfg(test)]
    pub fn new_without_link(id: Uuid, endpoint_name: String) -> Self {
        Self {
            id,
            endpoint_name,
            link: None,
        }
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
        self.clients.get(id).map(|client| ClientInfo {
            id: client.id,
            endpoint_name: client.endpoint_name.clone(),
        })
    }

    pub fn list_all_clients(&self) -> Vec<ClientInfo> {
        self.clients
            .values()
            .map(|client| ClientInfo {
                id: client.id,
                endpoint_name: client.endpoint_name.clone(),
            })
            .collect()
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

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn create_client(id: Uuid, endpoint_name: &str) -> Client {
        Client::new_without_link(id, endpoint_name.to_string())
    }

    #[test]
    fn test_subscribe_client() {
        let mut manager = ClientManager::new(2);
        let client1 = create_client(Uuid::new_v4(), "endpoint1");
        let client2 = create_client(Uuid::new_v4(), "endpoint2");

        assert!(manager.subscribe_client(client1).is_ok());
        assert!(manager.subscribe_client(client2).is_ok());
        assert_eq!(manager.get_count(), 2);
    }

    #[test]
    fn test_subscribe_client_exceeds_max() {
        let mut manager = ClientManager::new(1);
        let client1 = create_client(Uuid::new_v4(), "endpoint1");
        let client2 = create_client(Uuid::new_v4(), "endpoint2");

        assert!(manager.subscribe_client(client1).is_ok());
        assert!(manager.subscribe_client(client2).is_err());
        assert_eq!(manager.get_count(), 1);
    }

    #[tokio::test]
    async fn test_cancel_client() {
        let mut manager = ClientManager::new(1);
        let client_id = Uuid::new_v4();
        let client = create_client(client_id, "endpoint");

        manager.subscribe_client(client).unwrap();
        manager.cancel_client(&client_id, &None).await;

        assert_eq!(manager.get_count(), 0);
    }

    #[test]
    fn test_remove_client() {
        let mut manager = ClientManager::new(1);
        let client_id = Uuid::new_v4();
        let client = create_client(client_id, "endpoint");

        manager.subscribe_client(client).unwrap();
        manager.remove_client(&client_id);

        assert_eq!(manager.get_count(), 0);
    }

    #[test]
    fn test_get_info() {
        let mut manager = ClientManager::new(1);
        let client_id = Uuid::new_v4();
        let client = create_client(client_id, "endpoint");

        manager.subscribe_client(client).unwrap();
        let info = manager.get_info(&client_id).unwrap();

        assert_eq!(info.id, client_id);
        assert_eq!(info.endpoint_name, "endpoint");
    }

    #[test]
    fn test_list_all_clients() {
        let mut manager = ClientManager::new(2);
        let client1 = create_client(Uuid::new_v4(), "endpoint1");
        let client2 = create_client(Uuid::new_v4(), "endpoint2");

        manager.subscribe_client(client1).unwrap();
        manager.subscribe_client(client2).unwrap();

        let clients = manager.list_all_clients();
        assert_eq!(clients.len(), 2);
    }
}
