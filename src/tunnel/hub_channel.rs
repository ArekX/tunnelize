use std::sync::Arc;

use log::debug;
use tokio::sync::mpsc::Receiver;
use tokio_util::sync::CancellationToken;

use super::{messages::ChannelMessage, services::Services};

pub async fn start(
    mut channel_rx: Receiver<ChannelMessage>,
    services: Arc<Services>,
    cancel_token: CancellationToken,
) {
    loop {
        let message: ChannelMessage;

        tokio::select! {
                channel_message = channel_rx.recv() => {
                    message = match channel_message {
                        Some(message) => message,
                        None => {
                            debug!("Channel stopped.");
                            return;
                        }
                    };
                },
            _ = cancel_token.cancelled() => {
                debug!("Channel stopped.");
                return;
            }
        }

        debug!("Received message: {:?}", message);
    }
}
