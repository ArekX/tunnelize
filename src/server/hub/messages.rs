use serde::{Deserialize, Serialize};

use crate::tunnel::messages::TunnelMessage;

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
