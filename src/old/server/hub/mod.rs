use std::collections::HashMap;

mod hub_channel;
mod hub_server;
mod services;

use messages::HubChannelMessage;
use serde::{Deserialize, Serialize};
use services::Services;
use tokio::io::Result;
use tokio::sync::mpsc::{Receiver, Sender};

pub mod messages;
pub mod requests;

pub use services::HubService;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HubConfiguration {
    pub server_port: u16,
    pub max_tunnel_input_wait: u16,
}

pub async fn start_hub_server(
    hub_tx: Sender<HubChannelMessage>,
    hub_rx: Receiver<HubChannelMessage>,
    service_defs: HashMap<String, HubService>,
    config: HubConfiguration,
) -> Result<()> {
    let services = Services::create(service_defs, config, hub_tx);

    let hub_server = {
        let services = services.clone();
        tokio::spawn(async move {
            hub_server::start(services)
                .await
                .expect("Tcp Server Failed");
        })
    };

    let channel_listener = tokio::spawn(async move {
        hub_channel::start(services, hub_rx)
            .await
            .expect("Channel listener failed");
    });

    tokio::try_join!(hub_server, channel_listener)?;

    Ok(())
}
