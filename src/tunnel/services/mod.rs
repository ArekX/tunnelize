use std::sync::Arc;

use proxy_manager::ProxyManager;
use tokio::sync::{Mutex, MutexGuard};
use tunnel_data::TunnelData;
use uuid::Uuid;

use super::configuration::TunnelConfiguration;

mod proxy_manager;
mod tunnel_data;

pub struct Services {
    tunnel_data: Mutex<TunnelData>,
    proxy_manager: Mutex<ProxyManager>,
    config: Arc<TunnelConfiguration>,
}

impl Services {
    pub fn new(config: TunnelConfiguration) -> Self {
        Self {
            tunnel_data: Mutex::new(TunnelData::new()),
            proxy_manager: Mutex::new(ProxyManager::new()),
            config: Arc::new(config),
        }
    }

    pub async fn get_tunnel_data(&self) -> MutexGuard<TunnelData> {
        self.tunnel_data.lock().await
    }

    pub async fn get_proxy_manager(&self) -> MutexGuard<ProxyManager> {
        self.proxy_manager.lock().await
    }

    pub fn get_config(&self) -> Arc<TunnelConfiguration> {
        self.config.clone()
    }
}
