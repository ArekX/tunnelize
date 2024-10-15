use std::sync::Arc;

use proxy_manager::ProxyManager;
use tokio::sync::{Mutex, MutexGuard};

use super::configuration::TunnelConfiguration;

mod proxy_manager;

pub struct Services {
    proxy_manager: Mutex<ProxyManager>,
    config: Arc<TunnelConfiguration>,
}

impl Services {
    pub fn new(config: TunnelConfiguration) -> Self {
        Self {
            proxy_manager: Mutex::new(ProxyManager::new(&config)),
            config: Arc::new(config),
        }
    }

    pub async fn get_proxy_manager(&self) -> MutexGuard<ProxyManager> {
        self.proxy_manager.lock().await
    }

    pub fn get_config(&self) -> Arc<TunnelConfiguration> {
        self.config.clone()
    }
}
