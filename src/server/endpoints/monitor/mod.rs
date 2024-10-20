use std::sync::Arc;

use configuration::MonitorEndpointConfig;
use serde::{Deserialize, Serialize};
use tokio::io::Result;

use crate::{common::channel::RequestReceiver, server::services::Services};

use super::messages::EndpointChannelRequest;

pub mod configuration;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HttpEndpointInfo {
    assigned_url: String,
}

pub async fn start(
    services: Arc<Services>,
    name: String,
    config: MonitorEndpointConfig,
    mut channel_rx: RequestReceiver<EndpointChannelRequest>,
) -> Result<()> {
    Ok(())
}
