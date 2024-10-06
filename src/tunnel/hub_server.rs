use std::sync::Arc;

use tokio::io::Result;
use tokio::sync::mpsc::Sender;
use tokio_util::sync::CancellationToken;

use super::{messages::ChannelMessage, services::Services};

pub async fn start(channel_tx: Sender<ChannelMessage>, services: Arc<Services>) {
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(100)).await;
    }
}
