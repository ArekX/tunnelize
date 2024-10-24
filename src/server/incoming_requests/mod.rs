use std::{net::SocketAddr, sync::Arc};

use crate::{common::connection::ConnectionStream, create_data_enum};

use super::services::Services;

mod init_link;
mod init_tunnel;
mod monitoring_request;

pub use init_link::{InitLinkRequest, InitLinkResponse};
pub use init_tunnel::{InitTunelRequest, InitTunnelResponse, InputProxy, ProxySession};
pub use monitoring_request::{ProcessMonitoringRequest, ProcessMonitoringResponse};

create_data_enum!(ServerRequestMessage, {
    InitTunelRequest -> InitTunnelResponse,
    InitLinkRequest -> InitLinkResponse,
    ProcessMonitoringRequest -> ProcessMonitoringResponse
});

pub async fn handle(
    services: Arc<Services>,
    stream: ConnectionStream,
    address: SocketAddr,
    message: ServerRequestMessage,
) {
    match message {
        ServerRequestMessage::InitTunelRequest(request) => {
            init_tunnel::process(services, request, stream).await
        }
        ServerRequestMessage::InitLinkRequest(request) => {
            init_link::process(services, request, stream).await
        }
        ServerRequestMessage::ProcessMonitoringRequest(request) => {
            monitoring_request::process(services, request, stream, address).await
        }
    }
}
