use std::sync::Arc;

use crate::common::{connection::ConnectionStream, request::DataRequest};

use super::messages::ServerRequestMessage;
use super::services::Services;

mod auth_tunnel;

pub use auth_tunnel::{handle_auth_tunnel, AuthTunelRequest, AuthTunnelResponse};

pub async fn handle(
    services: Arc<Services>,
    stream: ConnectionStream,
    message: ServerRequestMessage,
) {
    match message {
        ServerRequestMessage::AuthTunnel(request) => {
            handle_auth_tunnel(services, DataRequest::new(request, stream)).await
        }
        _ => {}
    }
}
