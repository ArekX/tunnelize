use std::sync::Arc;

use bfp_manager::BfpManager;
use chrono::Utc;
use client_manager::ClientManager;
use endpoint_manager::EndpointManager;
use events::ServiceEvent;
use link_manager::LinkManager;
use tokio::sync::{Mutex, MutexGuard};
use tokio_util::sync::CancellationToken;
use tunnel_manager::TunnelManager;

use super::configuration::ServerConfiguration;

mod bfp_manager;
mod client_manager;
mod endpoint_manager;
pub mod events;
mod link_manager;
mod tunnel_manager;

pub use client_manager::{Client, ClientInfo};
pub use endpoint_manager::EndpointInfo;
pub use link_manager::LinkInfo;
pub use tunnel_manager::TunnelInfo;

pub trait HandleServiceEvent {
    async fn handle_event(&mut self, event: &ServiceEvent);
}

pub struct Services {
    client_manager: Mutex<ClientManager>,
    tunnel_manager: Mutex<TunnelManager>,
    endpoint_manager: Mutex<EndpointManager>,
    link_manager: Mutex<LinkManager>,
    bfp_manager: Mutex<BfpManager>,
    config: Arc<ServerConfiguration>,
    cancel_token: CancellationToken,
    start_time: i64,
}

impl Services {
    pub fn new(config: ServerConfiguration, cancel_token: CancellationToken) -> Self {
        Self {
            client_manager: Mutex::new(ClientManager::new()),
            tunnel_manager: Mutex::new(TunnelManager::new()),
            endpoint_manager: Mutex::new(EndpointManager::new()),
            link_manager: Mutex::new(LinkManager::new()),
            bfp_manager: Mutex::new(BfpManager::new()),
            config: Arc::new(config),
            start_time: Utc::now().timestamp(),
            cancel_token,
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

    pub async fn get_bfp_manager(&self) -> MutexGuard<BfpManager> {
        self.bfp_manager.lock().await
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

    pub fn get_uptime(&self) -> String {
        let uptime_seconds = Utc::now().timestamp() - self.start_time;
        let days = uptime_seconds / 86400;
        let hours = (uptime_seconds % 86400) / 3600;
        let minutes = (uptime_seconds % 3600) / 60;
        let seconds = uptime_seconds % 60;

        format!(
            "{} days, {} hours, {} minutes, {} seconds",
            days, hours, minutes, seconds
        )
    }

    pub fn get_cancel_token(&self) -> CancellationToken {
        self.cancel_token.clone()
    }
}
