use std::collections::HashMap;

use messages::HubMessage;
use serde::{Deserialize, Serialize};
use services::Services;
use tokio::io::Result;
use tokio::sync::mpsc::{Receiver, Sender};

pub mod messages;
pub mod requests;

mod hub_channel;
mod services;
mod tunnel_server;

pub use services::HubService;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HubConfiguration {
    pub server_port: u16,
    pub max_tunnel_input_wait: u16,
}

pub async fn start_hub_server(
    hub_tx: Sender<HubMessage>,
    hub_rx: Receiver<HubMessage>,
    service_defs: HashMap<String, HubService>,
    config: HubConfiguration,
) -> Result<()> {
    let services = Services::create(service_defs, config, hub_tx);

    let hub_server_services = services.clone();
    let hub_server = tokio::spawn(async move {
        tunnel_server::start(hub_server_services)
            .await
            .expect("Tcp Server Failed");
    });

    let channel_services = services.clone();
    let channel_listener = tokio::spawn(async move {
        hub_channel::start(channel_services, hub_rx)
            .await
            .expect("Channel listener failed");
    });

    tokio::try_join!(hub_server, channel_listener)?;

    Ok(())
}
