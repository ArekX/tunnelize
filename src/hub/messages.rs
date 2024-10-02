use serde::{Deserialize, Serialize};

use crate::http::messages::HttpTunnelMessage;

#[derive(Serialize, Deserialize)]
pub enum TunnelServerMessage {
    Monitor(MonitorMessage),
    Tunnel(TunnelMessage),
}

//tunnel -> hub -> service -> hub -> tunnel

pub enum HubMessage {
    Test(String),
    Tunnel(TunnelMessage),
}

#[derive(Serialize, Deserialize)]
pub enum MonitorMessage {
    ListServices,
}

#[derive(Serialize, Deserialize)]
pub struct TunnelMessage {
    pub service_name: String,
    pub data: TunnelMessageData,
}

#[derive(Serialize, Deserialize)]
pub enum TunnelMessageData {
    Http(HttpTunnelMessage),
}
