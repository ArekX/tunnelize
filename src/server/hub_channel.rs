use std::sync::Arc;

use log::debug;
use tokio::sync::mpsc::Receiver;

use super::{messages::ChannelMessage, services::Services};

pub async fn start(mut channel_rx: Receiver<ChannelMessage>, services: Arc<Services>) {
    loop {
        let message = match channel_rx.recv().await {
            Some(message) => message,
            None => {
                debug!("Channel stopped.");
                return;
            }
        };
    }
}
