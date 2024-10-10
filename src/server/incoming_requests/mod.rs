use std::sync::Arc;

use crate::common::{connection::ConnectionStream, request::DataRequest};

use super::services::Services;
use auth_link::handle_auth_link;
use auth_tunnel::handle_auth_tunnel;
use serde::{Deserialize, Serialize};

mod auth_link;
mod auth_tunnel;

pub use auth_link::{AuthLinkRequest, AuthLinkResponse};
pub use auth_tunnel::{AuthTunelRequest, AuthTunnelResponse};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ServerRequestMessage {
    AuthTunnel(AuthTunelRequest),
    AuthLink(AuthLinkRequest),
}

pub async fn handle(
    services: Arc<Services>,
    stream: ConnectionStream,
    message: ServerRequestMessage,
) {
    match message {
        ServerRequestMessage::AuthTunnel(request) => {
            handle_auth_tunnel(services, DataRequest::new(request, stream)).await
        }
        ServerRequestMessage::AuthLink(request) => {
            handle_auth_link(services, DataRequest::new(request, stream)).await
        }
    }
}
