use configuration::ServerConfiguration;
use log::{debug, info};
use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::sync::Arc;
use tokio::io::Result;

use tokio::sync::mpsc;

use messages::ChannelMessage;
use services::Services;
use tokio_util::sync::CancellationToken;

use crate::common::tasks::start_cancel_listener;

mod configuration;
mod endpoints;
mod hub_channel;
mod hub_server;
mod messages;
mod services;

pub async fn start() -> Result<()> {
    let configuration = ServerConfiguration {
        server_port: 3456,
        admin_key: None,
        max_tunnel_input_wait: 30,
        endpoint_key: None,
        endpoints: HashMap::new(),
    }; // TODO: This should be a parameter in start

    let (channel_tx, channel_rx) = mpsc::channel::<ChannelMessage>(100);
    let services = Arc::new(Services::new(configuration, channel_tx));

    let cancel_token = CancellationToken::new();

    let channel_future = {
        let services = services.clone();
        let cancel_token = cancel_token.clone();
        tokio::spawn(async move { hub_channel::start(channel_rx, services, cancel_token).await })
    };

    let server_future = {
        let services = services.clone();
        let cancel_token = cancel_token.clone();
        tokio::spawn(async move { hub_server::start(services, cancel_token).await })
    };

    let cancel_future = tokio::spawn(async move { start_cancel_listener(cancel_token).await });

    match tokio::try_join!(channel_future, server_future, cancel_future) {
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
