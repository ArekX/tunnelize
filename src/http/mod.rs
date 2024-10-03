use std::sync::Arc;

use log::{debug, info};
use serde::{Deserialize, Serialize};
use services::Services;
use tokio::sync::Mutex;

use tokio::{io::Result, sync::mpsc::Receiver, sync::mpsc::Sender};

use crate::hub::requests::{ServiceRequestData, ServiceResponse};
use crate::hub::{messages::HubChannelMessage, requests::ServiceRequest};

mod client_list;
mod host_list;
mod http_channel;
mod http_protocol;
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

pub async fn start_http_service(
    config: HttpServerConfig,
    service_rx: Receiver<ServiceRequest>,
    hub_tx: Sender<HubChannelMessage>,
) -> Result<()> {
    let services = Services::create(config.clone(), hub_tx);

    let client_task = {
        let services = services.clone();

        tokio::spawn(async move {
            http_server::start(services).await;
        })
    };

    let channel_task = tokio::spawn(async move {
        http_channel::start(services, service_rx).await;
    });

    tokio::try_join!(channel_task, client_task)?;

    Ok(())
}

pub async fn start_http_tunnel_service(
    server_address: String,
    config: HttpTunnelConfig,
) -> Result<()> {
    tunnel_client::start_client(server_address, config).await?;
    Ok(())
}
