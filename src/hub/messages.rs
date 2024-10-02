use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

#[derive(Serialize, Deserialize)]
pub enum HubTcpMessage {
    Monitor(MonitorMessage),
    Tunnel(TunnelMessage),
}

//tunnel -> hub -> service -> hub -> tunnel

#[derive(Serialize, Deserialize)]
pub struct TunnelMessage {
    pub service_name: String,
}

#[derive(Serialize, Deserialize)]
pub enum MonitorMessage {
    ListServices,
}

pub enum HubMessage {
    Test(String),
}
