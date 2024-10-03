use serde::{Deserialize, Serialize};

use super::super::http::messages::HttpTunnelMessage;

#[derive(Serialize, Deserialize)]
pub enum HubServerMessage {
    Monitor(MonitorMessage),
    Tunnel(TunnelMessage),
}

//tunnel -> hub -> service -> hub -> tunnel

pub enum HubChannelMessage {
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