use std::sync::Arc;

use client_manager::ClientManager;
use endpoint_manager::EndpointManager;
use events::ServiceEvent;
use link_manager::LinkManager;
use tokio::sync::{Mutex, MutexGuard};
use tunnel_manager::TunnelManager;

use super::configuration::ServerConfiguration;

mod client_manager;
mod endpoint_manager;
pub mod events;
mod link_manager;
mod tunnel_manager;

pub use client_manager::Client;

pub trait HandleServiceEvent {
    async fn handle_event(&mut self, event: &ServiceEvent);
}

pub struct Services {
    client_manager: Mutex<ClientManager>,
    tunnel_manager: Mutex<TunnelManager>,
    endpoint_manager: Mutex<EndpointManager>,
    link_manager: Mutex<LinkManager>,
    config: Arc<ServerConfiguration>,
}

impl Services {
    pub fn new(config: ServerConfiguration) -> Self {
        Self {
            client_manager: Mutex::new(ClientManager::new()),
            tunnel_manager: Mutex::new(TunnelManager::new()),
            endpoint_manager: Mutex::new(EndpointManager::new()),
            link_manager: Mutex::new(LinkManager::new()),
            config: Arc::new(config),
        }
    }

    pub async fn get_client_manager(&self) -> MutexGuard<ClientManager> {
        self.client_manager.lock().await
    }

    pub async fn get_tunnel_manager(&self) -> MutexGuard<TunnelManager> {
        self.tunnel_manager.lock().await
    }

    pub async fn get_endpoint_manager(&self) -> MutexGuard<EndpointManager> {
        self.endpoint_manager.lock().await
    }

    pub async fn get_link_manager(&self) -> MutexGuard<LinkManager> {
        self.link_manager.lock().await
    }

    pub async fn push_event(&self, event: ServiceEvent) {
        self.get_tunnel_manager().await.handle_event(&event).await;
        self.get_endpoint_manager().await.handle_event(&event).await;
        self.get_client_manager().await.handle_event(&event).await;
        self.get_link_manager().await.handle_event(&event).await;
    }

    pub fn get_config(&self) -> Arc<ServerConfiguration> {
        self.config.clone()
    }
}
