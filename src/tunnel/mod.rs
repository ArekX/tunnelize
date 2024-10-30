use configuration::{Encryption, ProxyConfiguration, TunnelConfiguration, TunnelProxy};
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
mod outgoing_requests;
mod services;

fn get_configuration() -> TunnelConfiguration {
    TunnelConfiguration {
        name: Some("test".to_string()),
        server_address: "127.0.0.1".to_string(),
        server_port: 3456,
        encryption: Encryption::Tls {
            cert: "certs/ca.crt".to_string(),
        },
        tunnel_key: None,
        monitor_key: Some("key".to_string()),
        proxies: vec![
            TunnelProxy {
                endpoint_name: "http".to_string(),
                address: "0.0.0.0".to_string(),
                port: 8081,
                config: ProxyConfiguration::Http {
                    desired_name: Some("test".to_string()),
                },
            },
            TunnelProxy {
                endpoint_name: "tcp".to_string(),
                address: "0.0.0.0".to_string(),
                port: 8080,
                config: ProxyConfiguration::Tcp { desired_port: None },
            },
            TunnelProxy {
                endpoint_name: "udp".to_string(),
                address: "0.0.0.0".to_string(),
                port: 8089,
                config: ProxyConfiguration::Udp { desired_port: None },
            },
        ],
    }
}

pub async fn process_monitor_command(command: MonitorCommands) -> Result<()> {
    outgoing_requests::process_monitor_request(get_configuration(), command).await?;

    Ok(())
}

pub async fn process_get_tunnel_config() -> Result<()> {
    let data = outgoing_requests::get_tunnel_config(get_configuration()).await?;

    debug!("Received tunnel config: {:?}", data);

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
