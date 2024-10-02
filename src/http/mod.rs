use std::sync::Arc;

use log::{debug, info};
use serde::{Deserialize, Serialize};
use services::Services;
use tokio::sync::Mutex;

use tokio::{io::Result, sync::mpsc::Receiver, sync::mpsc::Sender};

use crate::hub::requests::{ServiceRequestData, ServiceResponse};
use crate::hub::{messages::HubMessage, requests::ServiceRequest};

mod client_list;
mod host_list;
mod http_handler;
mod http_server;
pub mod messages;
mod services;
mod tunnel_client;
mod tunnel_list;
mod tunnel_server;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HttpServerConfig {
    pub client_port: u16,
    pub max_client_input_wait: u16,
    pub tunnel_auth_key: Option<String>,
    pub host_template: String,
    pub tunnel_url_template: String,
    pub allow_custom_hostnames: bool,
}

impl Default for HttpServerConfig {
    fn default() -> Self {
        HttpServerConfig {
            client_port: 3457,
            max_client_input_wait: 10,
            tunnel_auth_key: None,
            tunnel_url_template: "http://{hostname}:3457".to_string(),
            host_template: "t-{name}.localhost".to_string(),
            allow_custom_hostnames: true,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HttpTunnelConfig {
    pub proxies: Vec<TunnelProxy>,
    pub tunnel_auth_key: Option<String>,
    pub client_authorization: Option<ClientAuthorizeUser>,
}

impl Default for HttpTunnelConfig {
    fn default() -> Self {
        Self {
            proxies: vec![TunnelProxy {
                desired_name: Some("8000".to_string()),
                forward_address: "0.0.0.0:8000".to_owned(),
            }],
            tunnel_auth_key: None,
            client_authorization: None,
        }
    }
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

pub fn start_tunnel_task(services: Arc<Services>) -> tokio::task::JoinHandle<()> {
    info!("Starting tunnel listener");

    tokio::spawn(async move {
        tunnel_server::start_tunnel_server(services).await;
    })
}

pub fn start_client_task(services: Arc<Services>) -> tokio::task::JoinHandle<()> {
    info!("Starting client listener");
    tokio::spawn(async move {
        http_server::start_http_server(services).await;
    })
}

pub async fn start_http_server(
    config: HttpServerConfig,
    mut service_rx: Receiver<ServiceRequest>,
    hub_tx: Sender<HubMessage>,
) -> Result<()> {
    let services = Services::create(config.clone());

    // let tunnel_task = start_tunnel_task(services.clone());
    let client_task = start_client_task(services.clone());

    let channel_task = tokio::spawn(async move {
        loop {
            let request = match service_rx.recv().await {
                Some(request) => request,
                None => {
                    break;
                }
            };

            match request.data {
                ServiceRequestData::Http(_) => {
                    if let Err(_) = request
                        .response_tx
                        .send(ServiceResponse::Name("Works!".to_string()))
                    {
                        debug!("Failed to send response.");
                    }
                }
                _ => {
                    if let Err(_) = request
                        .response_tx
                        .send(ServiceResponse::Name("Unknown".to_string()))
                    {
                        debug!("Failed to send response.");
                    }
                }
            }
        }
    });

    tokio::try_join!(channel_task, client_task).unwrap();

    Ok(())
}

pub async fn start_http_tunnel(server_address: String, config: HttpTunnelConfig) -> Result<()> {
    tunnel_client::start_client(server_address, config).await?;
    Ok(())
}
