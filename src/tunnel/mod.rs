use configuration::TunnelConfiguration;
use log::debug;
use services::Services;
use std::io::Error;
use std::sync::Arc;
use tokio::io::Result;

use tokio_util::sync::CancellationToken;

use crate::common::cli::MonitorCommands;
use crate::common::tasks::start_cancel_listener;
use crate::configuration::load_configuration;

mod client;
pub mod configuration;
pub mod incoming_requests;
pub mod outgoing_requests;
mod services;

pub async fn process_monitor_command(
    command: MonitorCommands,
    configuration_file: Option<String>,
) -> Result<()> {
    outgoing_requests::process_monitor_request(load_configuration(configuration_file)?, command)
        .await?;

    Ok(())
}

pub async fn process_get_tunnel_config(configuration_file: Option<String>) -> Result<()> {
    let data =
        outgoing_requests::get_tunnel_config(load_configuration(configuration_file)?).await?;

    debug!("Received tunnel config: {:?}", data);

    Ok(())
}

pub async fn start(configuration_file: Option<String>) -> Result<()> {
    let configuration: TunnelConfiguration = load_configuration(configuration_file)?;

    let services = Arc::new(Services::new(configuration));
    let cancel_token = CancellationToken::new();

    let server_future = {
        let services = services.clone();
        let cancel_token = cancel_token.clone();
        tokio::spawn(async move {
            let result = client::start(services, cancel_token.clone()).await;
            if let Err(ref e) = result {
                debug!("Error starting tunnel client: {:?}", e);
            }

            cancel_token.cancel();

            result
        })
    };

    let cancel_future = tokio::spawn(async move { start_cancel_listener(cancel_token).await });

    match tokio::try_join!(server_future, cancel_future) {
        Ok((result, _)) => match result {
            Ok(_) => {
                println!("Tunnel stopped.");
                Ok(())
            }
            Err(_) => {
                println!("Error occurred while running the tunnel");
                Err(Error::other(
                    "Error occurred in tunnel run.",
                ))
            }
        },
        Err(_) => Err(Error::other(
            "Error occurred in tunnel run.",
        )),
    }
}
