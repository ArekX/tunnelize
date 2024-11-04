use super::{
    client_host::ClientHost, configuration::UdpEndpointConfig, messages::UdpChannelRequest,
    tunnel_host::TunnelHost,
};
use crate::{common::channel::RequestSender, server::services::Services as MainServices};
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};
use tokio_util::sync::CancellationToken;

pub struct UdpServices {
    config: Arc<UdpEndpointConfig>,
    leaf_hub_tx: RequestSender<UdpChannelRequest>,
    tunnel_host: Arc<Mutex<TunnelHost>>,
    client_host: Arc<Mutex<ClientHost>>,
    cancel_token: CancellationToken,
    main_services: Arc<MainServices>,
}

impl UdpServices {
    pub fn new(
        config: UdpEndpointConfig,
        main_services: Arc<MainServices>,
        leaf_hub_tx: RequestSender<UdpChannelRequest>,
    ) -> Self {
        let cancel_token = main_services.get_cancel_token();
        let tunnel_host = Arc::new(Mutex::new(TunnelHost::new(&config)));
        let client_host = Arc::new(Mutex::new(ClientHost::new()));

        Self {
            config: Arc::new(config),
            tunnel_host,
            client_host,
            cancel_token,
            main_services,
            leaf_hub_tx,
        }
    }

    pub fn get_leaf_hub_tx(&self) -> RequestSender<UdpChannelRequest> {
        self.leaf_hub_tx.clone()
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
