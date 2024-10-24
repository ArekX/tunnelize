use std::{net::SocketAddr, sync::Arc};

use serde::{Deserialize, Serialize};

use crate::{
    common::{address, cli::MonitorCommands, connection::ConnectionStream},
    server::{
        configuration::ServerConfiguration,
        monitoring::{self, SystemInfo},
        services::Services,
    },
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessMonitoringRequest {
    pub command: MonitorCommands,
    pub monitor_key: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProcessMonitoringResponse {
    SystemInfo(SystemInfo),
    Rejected { reason: String },
}

async fn has_monitoring_access(
    request: &ProcessMonitoringRequest,
    config: &Arc<ServerConfiguration>,
) -> bool {
    if let Some(config_monitor_key) = config.monitor_key.as_ref() {
        if let Some(request_monitor_key) = request.monitor_key.as_ref() {
            return config_monitor_key == request_monitor_key;
        }

        return false;
    }

    true
}

pub async fn process(
    services: Arc<Services>,
    request: ProcessMonitoringRequest,
    mut response_stream: ConnectionStream,
    address: SocketAddr,
) {
    let config = services.get_config();

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

    if !has_monitoring_access(&request, &config).await {
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
    }
}
