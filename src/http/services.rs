use std::sync::Arc;

use tokio::sync::{Mutex, MutexGuard};

use super::{
    client_list::ClientList, host_list::HostList, tunnel_list::TunnelList, HttpServerConfig,
};

pub struct Services {
    host_service: Mutex<HostList>,
    tunnel_service: Mutex<TunnelList>,
    client_service: Mutex<ClientList>,
    config: Arc<HttpServerConfig>,
}

impl Services {
    pub fn create(config: HttpServerConfig) -> Arc<Self> {
        Arc::new(Self {
            host_service: Mutex::new(HostList::new(
                config.host_template.clone(),
                config.allow_custom_hostnames,
            )),
            tunnel_service: Mutex::new(TunnelList::new()),
            client_service: Mutex::new(ClientList::new()),
            config: Arc::new(config),
        })
    }

    pub async fn get_host_service(&self) -> MutexGuard<HostList> {
        self.host_service.lock().await
    }

    pub async fn get_tunnel_service(&self) -> MutexGuard<TunnelList> {
        self.tunnel_service.lock().await
    }

    pub async fn get_client_service(&self) -> MutexGuard<ClientList> {
        self.client_service.lock().await
    }

    pub fn get_config(&self) -> Arc<HttpServerConfig> {
        self.config.clone()
    }
}
