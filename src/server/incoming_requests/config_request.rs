use std::{sync::Arc, vec};

use serde::{Deserialize, Serialize};

use crate::{
    common::connection::Connection,
    server::{
        configuration::EndpointConfiguration,
        endpoints::{
            http::configuration::HttpPublicEndpointConfig,
            tcp::configuration::TcpPublicEndpointConfig,
            udp::configuration::UdpPublicEndpointConfig,
        },
        services::Services,
    },
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessConfigRequest {
    pub tunnel_key: Option<String>,
    pub request: ConfigRequest,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ConfigRequest {
    GetPublicEndpointConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ProcessConfigResponse {
    GetPublicEndpointConfig(Vec<PublicEndpointConfig>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PublicEndpointConfig {
    pub name: String,
    pub config: PublicServerEndpointConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PublicServerEndpointConfig {
    Http(HttpPublicEndpointConfig),
    Tcp(TcpPublicEndpointConfig),
    Udp(UdpPublicEndpointConfig),
}

impl From<&EndpointConfiguration> for Option<PublicServerEndpointConfig> {
    fn from(config: &EndpointConfiguration) -> Self {
        match config {
            EndpointConfiguration::Http(config) => {
                Some(PublicServerEndpointConfig::Http(config.into()))
            }
            EndpointConfiguration::Tcp(config) => {
                Some(PublicServerEndpointConfig::Tcp(config.into()))
            }
            EndpointConfiguration::Udp(config) => {
                Some(PublicServerEndpointConfig::Udp(config.into()))
            }
            EndpointConfiguration::Monitoring(_) => None,
        }
    }
}

pub async fn process(
    services: Arc<Services>,
    request: ProcessConfigRequest,
    mut response_stream: Connection,
) {
    match &request.request {
        ConfigRequest::GetPublicEndpointConfig => {
            let endpoints = services.get_endpoint_manager().await.list_endpoints();

            let mut results: Vec<PublicEndpointConfig> = vec![];
            for endpoint in endpoints.iter() {
                if let Some(public_config) =
                    Option::<PublicServerEndpointConfig>::from(&endpoint.definition)
                {
                    results.push(PublicEndpointConfig {
                        name: endpoint.name.clone(),
                        config: public_config,
                    });
                }
            }

            response_stream
                .respond_message(&ProcessConfigResponse::GetPublicEndpointConfig(results))
                .await;
        }
    }
}
