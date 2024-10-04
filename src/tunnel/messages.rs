use serde::{Deserialize, Serialize};

use super::http::messages::HttpTunnelMessage;

#[derive(Serialize, Deserialize)]
pub struct TunnelMessage {
    pub service_name: String,
    pub data: TunnelMessageData,
}

#[derive(Serialize, Deserialize)]
pub enum TunnelMessageData {
    Http(HttpTunnelMessage),
}
