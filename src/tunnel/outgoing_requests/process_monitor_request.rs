use log::info;

use crate::{
    common::cli::MonitorCommands,
    server::incoming_requests::ProcessMonitoringRequest,
    tunnel::{client::create_server_connection, configuration::TunnelConfiguration},
};

pub async fn process_monitor_request(
    config: TunnelConfiguration,
    command: MonitorCommands,
) -> tokio::io::Result<()> {
    let mut connection = create_server_connection(&config).await?;

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
