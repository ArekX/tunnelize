use std::{sync::Arc, time::Duration};

use bfp_manager::BfpManager;
use chrono::Utc;
use client_manager::ClientManager;
use endpoint_manager::EndpointManager;
use events::ServiceEvent;
use link_manager::LinkManager;
use tokio::sync::{Mutex, MutexGuard};
use tokio_util::sync::CancellationToken;
use tunnel_manager::TunnelManager;
use uuid::Uuid;

use crate::common::channel::DataResponse;

use super::{
    configuration::ServerConfiguration,
    session::messages::{TunnelChannelRequest, TunnelChannelResponse},
};

pub const LINK_REQUEST_TIMEOUT: Duration = Duration::from_secs(60);

mod bfp_manager;
mod client_manager;
mod endpoint_manager;
pub mod events;
mod link_manager;
mod tunnel_manager;

pub use client_manager::{Client, ClientInfo, ClientLink, LinkHandoff};
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
            client_manager: Mutex::new(ClientManager::new(config.get_max_clients())),
            tunnel_manager: Mutex::new(TunnelManager::new()),
            endpoint_manager: Mutex::new(EndpointManager::new()),
            link_manager: Mutex::new(LinkManager::new()),
            bfp_manager: Mutex::new(BfpManager::new()),
            config: Arc::new(config),
            start_time: Utc::now().timestamp(),
            cancel_token,
        }
    }

    pub async fn get_client_manager(&self) -> MutexGuard<'_, ClientManager> {
        self.client_manager.lock().await
    }

    pub async fn get_tunnel_manager(&self) -> MutexGuard<'_, TunnelManager> {
        self.tunnel_manager.lock().await
    }

    pub async fn get_endpoint_manager(&self) -> MutexGuard<'_, EndpointManager> {
        self.endpoint_manager.lock().await
    }

    pub async fn get_link_manager(&self) -> MutexGuard<'_, LinkManager> {
        self.link_manager.lock().await
    }

    pub async fn get_bfp_manager(&self) -> MutexGuard<'_, BfpManager> {
        self.bfp_manager.lock().await
    }

    pub async fn send_tunnel_request<T: Into<TunnelChannelRequest> + DataResponse>(
        &self,
        tunnel_id: &Uuid,
        request: T,
    ) -> tokio::io::Result<T::Response>
    where
        T::Response: TryFrom<TunnelChannelResponse>,
    {
        let Some(tunnel_tx) = self.get_tunnel_manager().await.get_session_tx(tunnel_id) else {
            return Err(tokio::io::Error::new(
                tokio::io::ErrorKind::NotFound,
                format!("Tunnel session not found: {tunnel_id:?}"),
            ));
        };

        match tokio::time::timeout(LINK_REQUEST_TIMEOUT, tunnel_tx.request(request)).await {
            Ok(result) => result,
            Err(_) => Err(tokio::io::Error::new(
                tokio::io::ErrorKind::TimedOut,
                "Timed out waiting for tunnel to answer the request",
            )),
        }
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

        format!("{days} days, {hours} hours, {minutes} minutes, {seconds} seconds")
    }

    pub fn get_cancel_token(&self) -> CancellationToken {
        self.cancel_token.clone()
    }
}
