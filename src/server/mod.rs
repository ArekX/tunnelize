use configuration::ServerConfiguration;
use log::{debug, info};
use std::io::{Error, ErrorKind};
use std::sync::Arc;
use tokio::io::Result;

use services::Services;
use tokio_util::sync::CancellationToken;

use crate::common::tasks::start_cancel_listener;
use crate::configuration::load_configuration;

pub mod configuration;
pub mod endpoints;
mod hub_server;
pub mod incoming_requests;
mod monitoring;
mod services;
mod session;

pub async fn start(configuration_file: Option<String>) -> Result<()> {
    let configuration: ServerConfiguration = load_configuration(configuration_file)?;

    let cancel_token = CancellationToken::new();
    let services = Arc::new(Services::new(configuration, cancel_token.clone()));

    let server_future = {
        let services = services.clone();
        let cancel_token = cancel_token.clone();
        tokio::spawn(async move {
            if let Err(e) = hub_server::start(services, cancel_token.clone()).await {
                debug!("Error starting hub server: {:?}", e);
            }

            cancel_token.cancel();
        })
    };

    let cancel_future = tokio::spawn(async move { start_cancel_listener(cancel_token).await });

    match tokio::try_join!(server_future, cancel_future) {
        Ok(_) => {
            info!("Server stopped.");
            Ok(())
        }
        Err(_) => Err(Error::new(
            ErrorKind::Other,
            "Error occurred in server run.",
        )),
    }
}
