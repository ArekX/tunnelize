use std::{net::SocketAddr, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::{
    common::{cli::MonitorCommands, connection::Connection},
    server::{
        monitoring::{self, Records, SystemInfo},
        services::{ClientInfo, EndpointInfo, LinkInfo, Services, TunnelInfo},
    },
};

use super::access::{has_monitoring_access, has_tunnel_access};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessMonitoringRequest {
    pub command: MonitorCommands,
    pub monitor_key: Option<String>,
    pub tunnel_key: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProcessMonitoringResponse {
    SystemInfo(SystemInfo),
    ListEndpoints(Records<EndpointInfo>),
    ListClients(Records<ClientInfo>),
    GetClient(ClientInfo),
    ListTunnels(Records<TunnelInfo>),
    GetTunnel(TunnelInfo),
    TunnelDisconnected,
    ListLinks(Records<LinkInfo>),
    GetLink(LinkInfo),
    LinkDisconnected,
    Rejected { reason: String },
}

pub async fn process(
    services: Arc<Services>,
    request: ProcessMonitoringRequest,
    mut response_stream: Connection,
    address: SocketAddr,
) {
    let ip_address = address.ip();

    if services.get_bfp_manager().await.is_locked(&ip_address) {
        response_stream
            .respond_message(&ProcessMonitoringResponse::Rejected {
                reason: "Access denied. Too many attempts. Please try later.".to_string(),
            })
            .await;

        response_stream.shutdown().await;
        return;
    }

    if !has_tunnel_access(&services, request.tunnel_key.as_ref()) {
        services.get_bfp_manager().await.log_ip_attempt(&ip_address);
        response_stream
            .respond_message(&ProcessMonitoringResponse::Rejected {
                reason: "Access denied. Invalid monitor key.".to_string(),
            })
            .await;

        response_stream.shutdown().await;
        return;
    }

    if !has_monitoring_access(&services, request.monitor_key.as_ref()) {
        services.get_bfp_manager().await.log_ip_attempt(&ip_address);
        response_stream
            .respond_message(&ProcessMonitoringResponse::Rejected {
                reason: "Access denied. Invalid monitor key.".to_string(),
            })
            .await;

        response_stream.shutdown().await;
        return;
    }

    services
        .get_bfp_manager()
        .await
        .clear_ip_attempts(&ip_address);

    match request.command {
        MonitorCommands::SystemInfo => {
            response_stream
                .respond_message(&ProcessMonitoringResponse::SystemInfo(
                    monitoring::get_system_info(&services).await,
                ))
                .await;
        }
        MonitorCommands::ListTunnels => {
            response_stream
                .respond_message(&ProcessMonitoringResponse::ListTunnels(
                    monitoring::get_tunnel_list(&services).await.into(),
                ))
                .await;
        }
        MonitorCommands::GetTunnel { id } => {
            let Some(result) = monitoring::get_tunnel_info(&services, &id).await else {
                response_stream
                    .respond_message(&ProcessMonitoringResponse::Rejected {
                        reason: "Tunnel not found for this ID".to_string(),
                    })
                    .await;
                return;
            };

            response_stream
                .respond_message(&ProcessMonitoringResponse::GetTunnel(result))
                .await;
        }
        MonitorCommands::DisconnectTunnel { id } => {
            if let Err(error) = services.get_tunnel_manager().await.cancel_session(&id) {
                response_stream
                    .respond_message(&ProcessMonitoringResponse::Rejected { reason: error })
                    .await;
                return;
            };

            response_stream
                .respond_message(&ProcessMonitoringResponse::TunnelDisconnected)
                .await;
        }
        MonitorCommands::ListEndpoints => {
            response_stream
                .respond_message(&ProcessMonitoringResponse::ListEndpoints(
                    monitoring::get_endpoint_list(&services).await.into(),
                ))
                .await;
        }
        MonitorCommands::ListClients => {
            response_stream
                .respond_message(&ProcessMonitoringResponse::ListClients(
                    monitoring::get_client_list(&services).await.into(),
                ))
                .await;
        }
        MonitorCommands::GetClient { id } => {
            let Some(result) = monitoring::get_client_info(&services, &id).await else {
                response_stream
                    .respond_message(&ProcessMonitoringResponse::Rejected {
                        reason: "Client not found for this ID".to_string(),
                    })
                    .await;
                return;
            };

            response_stream
                .respond_message(&ProcessMonitoringResponse::GetClient(result))
                .await;
        }
        MonitorCommands::ListLinks => {
            response_stream
                .respond_message(&ProcessMonitoringResponse::ListLinks(
                    monitoring::get_link_list(&services).await.into(),
                ))
                .await;
        }
        MonitorCommands::GetLink { id } => {
            let Some(result) = monitoring::get_link_info(&services, &id).await else {
                response_stream
                    .respond_message(&ProcessMonitoringResponse::Rejected {
                        reason: "Link not found for this ID".to_string(),
                    })
                    .await;
                return;
            };

            response_stream
                .respond_message(&ProcessMonitoringResponse::GetLink(result))
                .await;
        }
        MonitorCommands::DisconnectLink { id } => {
            if let Err(error) = monitoring::disconnect_link(&services, &id).await {
                response_stream
                    .respond_message(&ProcessMonitoringResponse::Rejected { reason: error })
                    .await;
                return;
            };

            response_stream
                .respond_message(&ProcessMonitoringResponse::LinkDisconnected)
                .await;
        }
    }
}
