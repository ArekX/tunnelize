use std::sync::Arc;

use client_manager::ClientManager;
use tokio::sync::{mpsc::Sender, Mutex, MutexGuard};
use tunnel_manager::TunnelManager;

use super::{configuration::ServerConfiguration, messages::ChannelMessage};

mod client_manager;
mod tunnel_manager;

pub struct Services {
    client_manager: Mutex<ClientManager>,
    tunnel_manager: Mutex<TunnelManager>,
    config: Arc<ServerConfiguration>,
    hub_tx: Sender<ChannelMessage>,
}

impl Services {
    pub fn new(config: ServerConfiguration, hub_tx: Sender<ChannelMessage>) -> Self {
        Self {
            client_manager: Mutex::new(ClientManager::new()),
            tunnel_manager: Mutex::new(TunnelManager::new()),
            config: Arc::new(config),
            hub_tx,
        }
    }

    pub async fn get_client_manager(&self) -> MutexGuard<ClientManager> {
        self.client_manager.lock().await
    }

    pub async fn get_tunnel_manager(&self) -> MutexGuard<TunnelManager> {
        self.tunnel_manager.lock().await
    }

    pub fn get_config(&self) -> Arc<ServerConfiguration> {
        self.config.clone()
    }

    pub fn get_hub_tx(&self) -> Sender<ChannelMessage> {
        self.hub_tx.clone()
    }
}
