use std::sync::Arc;

use client_list::ClientList;
use host_list::HostList;
use log::info;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tunnel_list::TunnelList;

use crate::configuration::TunnelConfiguration;

mod client_list;
mod client_resolver;
mod host_list;
mod http_server;
mod messages;
mod tunnel_client;
mod tunnel_helper;
mod tunnel_list;
mod tunnel_server;

pub type TaskService<T> = Arc<Mutex<T>>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HttpServerConfig {
    pub client_port: u16,
    pub tunnel_port: u16,
    pub auth_key: Option<String>,
    pub host_template: String,
}

pub fn start_tunnel_task(
    host_service: TaskService<HostList>,
    tunnel_service: TaskService<TunnelList>,
    client_service: TaskService<ClientList>,
    config: HttpServerConfig,
) -> tokio::task::JoinHandle<()> {
    info!("Starting tunnel listener");
    tokio::spawn(async move {
        tunnel_server::start_tunnel_server(
            config.clone(),
            host_service.clone(),
            tunnel_service.clone(),
            client_service.clone(),
        )
        .await;
    })
}

pub fn start_client_task(
    host_service: TaskService<HostList>,
    tunnel_service: TaskService<TunnelList>,
    client_service: TaskService<ClientList>,
    config: HttpServerConfig,
) -> tokio::task::JoinHandle<()> {
    info!("Starting client listener");
    tokio::spawn(async move {
        http_server::start_http_server(
            config.clone(),
            host_service.clone(),
            tunnel_service.clone(),
            client_service.clone(),
        )
        .await;
    })
}

pub async fn start_http_server(config: HttpServerConfig) -> Result<(), std::io::Error> {
    let client_service: TaskService<ClientList> = Arc::new(Mutex::new(ClientList::new()));
    let host_service: TaskService<HostList> =
        Arc::new(Mutex::new(HostList::new(config.host_template.clone())));
    let tunnel_service: TaskService<TunnelList> = Arc::new(Mutex::new(TunnelList::new()));

    let tunnel_task = start_tunnel_task(
        host_service.clone(),
        tunnel_service.clone(),
        client_service.clone(),
        config.clone(),
    );
    let client_task = start_client_task(
        host_service.clone(),
        tunnel_service.clone(),
        client_service.clone(),
        config.clone(),
    );

    tokio::join!(tunnel_task, client_task).0?;

    Ok(())
}

pub async fn start_http_tunnel(config: TunnelConfiguration) -> Result<(), std::io::Error> {
    tunnel_client::start_client(config).await?;
    Ok(())
}
