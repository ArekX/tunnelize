use proxy_manager::ProxyManager;
use tokio::sync::Mutex;

mod proxy_manager;

pub struct Services {
    proxy_manager: Mutex<ProxyManager>,
}

impl Services {
    pub fn new() -> Self {
        Self {
            proxy_manager: Mutex::new(ProxyManager::new()),
        }
    }
}
