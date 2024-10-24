use configuration::{ProxyConfiguration, TunnelConfiguration, TunnelProxy};
use log::debug;
use services::Services;
use std::io::{Error, ErrorKind};
use std::sync::Arc;
use tokio::io::Result;

use tokio_util::sync::CancellationToken;

use crate::common::cli::MonitorCommands;
use crate::common::tasks::start_cancel_listener;

mod client;
pub mod configuration;
pub mod incoming_requests;
mod monitor;
mod outgoing_requests;
mod services;

fn get_configuration() -> TunnelConfiguration {
    TunnelConfiguration {
        name: Some("test".to_string()),
        server_address: "127.0.0.1:3456".to_string(),
        endpoint_key: None,
        monitor_key: Some("key".to_string()),
        proxies: vec![TunnelProxy {
            endpoint_name: "http".to_string(),
            forward_address: "0.0.0.0:8080".to_string(),
            config: ProxyConfiguration::Http {
                desired_name: Some("test".to_string()),
            },
        }],
    }
}

pub async fn process_monitor_command(command: MonitorCommands) -> Result<()> {
    monitor::process_monitor_request(get_configuration(), command).await?;

    Ok(())
}

pub async fn start() -> Result<()> {
    let configuration = get_configuration();

    let services = Arc::new(Services::new(configuration));
    let cancel_token = CancellationToken::new();

    let server_future = {
        let services = services.clone();
        let cancel_token = cancel_token.clone();
        tokio::spawn(async move {
            if let Err(e) = client::start(services, cancel_token.clone()).await {
                debug!("Error starting tunnel client: {:?}", e);
            }

            cancel_token.cancel();
        })
    };

    let cancel_future = tokio::spawn(async move { start_cancel_listener(cancel_token).await });

    match tokio::try_join!(server_future, cancel_future) {
        Ok(_) => {
            println!("Tunnel stopped.");
            Ok(())
        }
        Err(_) => Err(Error::new(
            ErrorKind::Other,
            "Error occurred in server run.",
        )),
    }
}
