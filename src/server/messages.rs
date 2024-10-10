use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::requests::AuthTunelRequest;

#[derive(Debug)]
pub enum ChannelMessage {}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ServerRequestMessage {
    AuthTunnel(AuthTunelRequest),
    AuthLinkRequest { tunnel_id: Uuid, session_id: Uuid },
}
