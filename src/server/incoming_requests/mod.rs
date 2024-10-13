use std::sync::Arc;

use crate::common::{connection::ConnectionStream, data_request::DataRequest};

use super::services::Services;
use init_link::process_init_link;
use init_tunnel::process_init_tunnel;
use serde::{Deserialize, Serialize};

mod init_link;
mod init_tunnel;

pub use init_link::{InitLinkRequest, InitLinkResponse};
pub use init_tunnel::{InitTunelRequest, InitTunnelResponse};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ServerRequestMessage {
    InitTunnel(InitTunelRequest),
    InitLink(InitLinkRequest),
}

pub async fn handle(
    services: Arc<Services>,
    stream: ConnectionStream,
    message: ServerRequestMessage,
) {
    match message {
        ServerRequestMessage::InitTunnel(request) => {
            process_init_tunnel(services, DataRequest::new(request, stream)).await
        }
        ServerRequestMessage::InitLink(request) => {
            process_init_link(services, DataRequest::new(request, stream)).await
        }
    }
}
