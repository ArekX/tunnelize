use log::info;

use crate::{
    common::cli::MonitorCommands, server::incoming_requests::ProcessMonitoringRequest,
    tunnel::configuration::TunnelConfiguration,
};

pub async fn process_monitor_request(
    config: TunnelConfiguration,
    command: MonitorCommands,
) -> tokio::io::Result<()> {
    let mut connection = config.create_tcp_client().await?;

    let response = connection
        .request_message(ProcessMonitoringRequest {
            command,
            monitor_key: config.monitor_key.clone(),
        })
        .await?;

    info!("Received monitoring response.");
    println!("{}", serde_json::to_string_pretty(&response)?);

    info!("Monitor request completed.");

    Ok(())
}
