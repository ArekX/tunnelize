use client_manager::ClientManager;
use tokio::sync::Mutex;
use tunnel_manager::TunnelManager;

mod client_manager;
mod tunnel_manager;

pub struct Services {
    client_manager: Mutex<ClientManager>,
    tunnel_manager: Mutex<TunnelManager>,
}

impl Services {
    pub fn new() -> Self {
        Self {
            client_manager: Mutex::new(ClientManager::new()),
            tunnel_manager: Mutex::new(TunnelManager::new()),
        }
    }
}
