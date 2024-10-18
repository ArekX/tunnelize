use std::sync::Arc;

use crate::{common::connection::ConnectionStream, create_data_enum};

use super::services::Services;
use init_link::process_init_link;
use init_tunnel::process_init_tunnel;
use serde::{Deserialize, Serialize};

mod init_link;
mod init_tunnel;

pub use init_link::{InitLinkRequest, InitLinkResponse};
pub use init_tunnel::{InitTunelRequest, InitTunnelResponse, InputProxy, ProxySession};

create_data_enum!(ServerRequestMessage, {
    InitTunelRequest -> InitTunnelResponse,
    InitLinkRequest -> InitLinkResponse
});

pub async fn handle(
    services: Arc<Services>,
    stream: ConnectionStream,
    message: ServerRequestMessage,
) {
    match message {
        ServerRequestMessage::InitTunelRequest(request) => {
            process_init_tunnel(services, request, stream).await
        }
        ServerRequestMessage::InitLinkRequest(request) => {
            process_init_link(services, request, stream).await
        }
    }
}
