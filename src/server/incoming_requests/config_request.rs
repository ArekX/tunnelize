use std::{net::SocketAddr, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::{
    common::connection::Connection,
    server::{configuration::PublicEndpointConfiguration, services::Services},
};

use super::access::has_tunnel_access;

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
    AccessDenied,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PublicEndpointConfig {
    pub name: String,
    pub config: PublicEndpointConfiguration,
}

pub async fn process(
    services: Arc<Services>,
    request: ProcessConfigRequest,
    mut response_stream: Connection,
    address: SocketAddr,
) {
    match &request.request {
        ConfigRequest::GetPublicEndpointConfig => {
            let ip_address = address.ip();

            if services.get_bfp_manager().await.is_locked(&ip_address) {
                response_stream
                    .respond_message(&ProcessConfigResponse::AccessDenied)
                    .await;
                response_stream.shutdown().await;
                return;
            }

            if !has_tunnel_access(&services, request.tunnel_key.as_ref()) {
                services.get_bfp_manager().await.log_ip_attempt(&ip_address);
                response_stream
                    .respond_message(&ProcessConfigResponse::AccessDenied)
                    .await;
                return;
            }

            services
                .get_bfp_manager()
                .await
                .clear_ip_attempts(&ip_address);

            let endpoints = services
                .get_endpoint_manager()
                .await
                .list_endpoints()
                .drain(0..)
                .map(|endpoint| PublicEndpointConfig {
                    name: endpoint.name.clone(),
                    config: endpoint.definition,
                })
                .collect();

            response_stream
                .respond_message(&ProcessConfigResponse::GetPublicEndpointConfig(endpoints))
                .await;
        }
    }
}
