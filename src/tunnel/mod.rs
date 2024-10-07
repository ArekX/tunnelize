use configuration::TunnelConfiguration;
use log::{debug, info};
use services::Services;
use std::io::{Error, ErrorKind};
use std::sync::Arc;
use tokio::io::Result;

use tokio_util::sync::CancellationToken;

use crate::common::tasks::start_cancel_listener;

mod configuration;
mod hub_client;
mod messages;
mod services;

pub async fn start() -> Result<()> {
    let configuration = TunnelConfiguration {
        server_host: "0.0.0.0:3456".to_string(),
        endpoint_key: None,
        admin_key: None,
        proxies: vec![],
    }; // TODO: This should be a parameter in start

    let services = Arc::new(Services::new(configuration));
    let cancel_token = CancellationToken::new();

    let server_future = {
        let services = services.clone();
        let cancel_token = cancel_token.clone();
        tokio::spawn(async move {
            if let Err(e) = hub_client::start(services, cancel_token).await {
                debug!("Error starting tunnel client: {:?}", e);
            }
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
