use std::sync::Arc;

use log::debug;
use tokio::io::Result;
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;

use super::{messages::ChannelMessage, services::Services};

pub async fn start(
    channel_tx: Sender<ChannelMessage>,
    services: Arc<Services>,
    cancel_token: CancellationToken,
) {
    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                debug!("Hub server stopped.");
                return;
            }
            _ = tokio::time::sleep(std::time::Duration::from_secs(100)) => {
                debug!("Hub server tick.");

            }
        }
    }
}
