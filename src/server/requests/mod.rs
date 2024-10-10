use std::sync::Arc;

use crate::common::connection::ConnectionStream;

use super::messages::ServerRequestMessage;
use super::services::Services;

mod auth_tunnel;

pub struct ServerRequest<T> {
    pub data: T,
    pub stream: ConnectionStream,
}

impl<T> ServerRequest<T> {
    pub fn new(stream: ConnectionStream, data: T) -> Self {
        Self { data, stream }
    }
}

pub use auth_tunnel::{process_auth_tunel_request, AuthTunelRequest};

pub async fn handle(
    services: Arc<Services>,
    stream: ConnectionStream,
    message: ServerRequestMessage,
) {
    match message {
        ServerRequestMessage::AuthTunnel(request) => {
            process_auth_tunel_request(services, ServerRequest::new(stream, request)).await;
        }
        _ => {}
    }
}
