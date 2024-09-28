use std::sync::Arc;

use client_list::ClientList;
use host_list::HostList;
use log::info;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tunnel_list::TunnelList;

mod client_list;
mod host_list;
mod http_handler;
mod http_server;
mod messages;
mod tunnel_client;
mod tunnel_list;
mod tunnel_server;

pub type TaskService<T> = Arc<Mutex<T>>;
pub type TaskData<T> = Arc<T>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HttpServerConfig {
    pub client_port: u16,
    pub tunnel_auth_key: Option<String>,
    pub host_template: String,
    pub tunnel_url_template: String,
    pub allow_custom_hostnames: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HttpTunnelConfig {
    pub proxies: Vec<TunnelProxy>,
    pub tunnel_auth_key: Option<String>,
    pub client_authorization: Option<ClientAuthorizeUser>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TunnelProxy {
    pub desired_name: Option<String>,
    pub forward_address: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClientAuthorizeUser {
    pub realm: Option<String>,
    pub username: String,
    pub password: String,
}

pub fn start_tunnel_task(
    host_service: TaskService<HostList>,
    tunnel_service: TaskService<TunnelList>,
    client_service: TaskService<ClientList>,
    config: TaskData<HttpServerConfig>,
    tunnel_port: u16,
) -> tokio::task::JoinHandle<()> {
    info!("Starting tunnel listener");
    tokio::spawn(async move {
        tunnel_server::start_tunnel_server(
            tunnel_port,
            config,
            host_service,
            tunnel_service,
            client_service,
        )
        .await;
    })
}

pub fn start_client_task(
    host_service: TaskService<HostList>,
    tunnel_service: TaskService<TunnelList>,
    client_service: TaskService<ClientList>,
    config_service: TaskData<HttpServerConfig>,
) -> tokio::task::JoinHandle<()> {
    info!("Starting client listener");
    tokio::spawn(async move {
        http_server::start_http_server(
            config_service,
            host_service,
            tunnel_service,
            client_service,
        )
        .await;
    })
}

pub async fn start_http_server(
    tunnel_port: u16,
    config: HttpServerConfig,
) -> Result<(), std::io::Error> {
    let host_service: TaskService<HostList> = Arc::new(Mutex::new(HostList::new(
        config.host_template.clone(),
        config.allow_custom_hostnames,
    )));
    let config_service: TaskData<HttpServerConfig> = Arc::new(config);
    let client_service: TaskService<ClientList> = Arc::new(Mutex::new(ClientList::new()));

    let tunnel_service: TaskService<TunnelList> = Arc::new(Mutex::new(TunnelList::new()));

    let tunnel_task = start_tunnel_task(
        host_service.clone(),
        tunnel_service.clone(),
        client_service.clone(),
        config_service.clone(),
        tunnel_port,
    );
    let client_task = start_client_task(
        host_service.clone(),
        tunnel_service.clone(),
        client_service.clone(),
        config_service.clone(),
    );

    tokio::join!(tunnel_task, client_task).0?;

    Ok(())
}

pub async fn start_http_tunnel(
    server_address: String,
    config: HttpTunnelConfig,
) -> Result<(), std::io::Error> {
    tunnel_client::start_client(server_address, config).await?;
    Ok(())
}
