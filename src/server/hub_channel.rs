use std::sync::Arc;

use super::{messages::ChannelMessage, services::Services};
use log::debug;
use tokio::io::Result;
use tokio::sync::mpsc::Receiver;
use tokio_util::sync::CancellationToken;

pub async fn start(
    mut channel_rx: Receiver<ChannelMessage>,
    services: Arc<Services>,
    cancel_token: CancellationToken,
) -> Result<()> {
    loop {
        let message: ChannelMessage;

        tokio::select! {
                channel_message = channel_rx.recv() => {
                    message = match channel_message {
                        Some(message) => message,
                        None => {
                            debug!("Channel stopped.");
                            return Ok(());
                        }
                    };
                },
            _ = cancel_token.cancelled() => {
                debug!("Channel stopped.");
                return Ok(());
            }
        }

        debug!("Received message: {:?}", message);
    }
}
