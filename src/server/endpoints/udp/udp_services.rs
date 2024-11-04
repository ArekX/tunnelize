use super::{client_host::ClientHost, configuration::UdpEndpointConfig, tunnel_host::TunnelHost};
use crate::server::services::Services as MainServices;
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};
use tokio_util::sync::CancellationToken;

pub struct UdpServices {
    config: Arc<UdpEndpointConfig>,
    name: String,
    tunnel_host: Arc<Mutex<TunnelHost>>,
    client_host: Arc<Mutex<ClientHost>>,
    cancel_token: CancellationToken,
    main_services: Arc<MainServices>,
}

impl UdpServices {
    pub fn new(config: UdpEndpointConfig, name: String, main_services: Arc<MainServices>) -> Self {
        let cancel_token = main_services.get_cancel_token();
        let tunnel_host = Arc::new(Mutex::new(TunnelHost::new(&config)));
        let client_host = Arc::new(Mutex::new(ClientHost::new()));

        Self {
            config: Arc::new(config),
            tunnel_host,
            name,
            client_host,
            cancel_token,
            main_services,
        }
    }

    pub fn get_endpoint_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_config(&self) -> Arc<UdpEndpointConfig> {
        self.config.clone()
    }

    pub async fn get_tunnel_host(&self) -> MutexGuard<TunnelHost> {
        self.tunnel_host.lock().await
    }

    pub async fn get_client_host(&self) -> MutexGuard<ClientHost> {
        self.client_host.lock().await
    }

    pub fn get_cancel_token(&self) -> CancellationToken {
        self.cancel_token.clone()
    }

    pub fn get_main_services(&self) -> Arc<MainServices> {
        self.main_services.clone()
    }
}
