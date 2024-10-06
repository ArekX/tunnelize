use log::{debug, info};
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
    let services = Arc::new(Services::new());
    let (channel_tx, channel_rx) = mpsc::channel::<ChannelMessage>(100);
    let cancel_token = CancellationToken::new();

    let channel_future = {
        let services = services.clone();
        let cancel_token = cancel_token.clone();
        tokio::spawn(async move { hub_channel::start(channel_rx, services, cancel_token).await })
    };

    let server_future = {
        let services = services.clone();
        let cancel_token = cancel_token.clone();
        tokio::spawn(async move { hub_server::start(channel_tx, services, cancel_token).await })
    };

    let cancel_future = tokio::spawn(async move { start_cancel_listener(cancel_token) });

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
