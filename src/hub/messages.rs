use crate::http::messages::HttpTunnelMessage;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum TunnelMessage {
    Cli(CliMessage),
    Http(HttpTunnelMessage),
}

#[derive(Serialize, Deserialize)]
pub enum CliMessage {
    ListServices,
}

pub enum HubMessage {
    Name(String),
}
