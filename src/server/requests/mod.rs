use std::sync::Arc;
use tokio::net::TcpStream;

use super::messages::ServerRequestMessage;
use super::services::Services;

mod auth_tunnel;

pub struct ServerRequest<T> {
    pub data: T,
    pub stream: TcpStream,
}

impl<T> ServerRequest<T> {
    pub fn new(stream: TcpStream, data: T) -> Self {
        Self { data, stream }
    }
}

pub use auth_tunnel::{process_auth_tunel_request, AuthTunelRequest};

pub async fn handle_server_message(
    services: Arc<Services>,
    stream: TcpStream,
    message: ServerRequestMessage,
) {
    match message {
        ServerRequestMessage::AuthTunnel(request) => {
            process_auth_tunel_request(services, ServerRequest::new(stream, request)).await;
        }
        _ => {}
    }
}
