use std::sync::Arc;

use log::info;
use tokio::io::{self, Result};

use crate::common::connection::ConnectionStream;
use crate::server::incoming_requests::{
    InitTunelRequest, InitTunnelResponse, ServerRequestMessage,
};

use crate::tunnel::configuration::TunnelConfiguration;

pub async fn authenticate_tunnel(
    config: &Arc<TunnelConfiguration>,
    server: &mut ConnectionStream,
) -> Result<()> {
    let auth_response: InitTunnelResponse = server
        .request_message(&ServerRequestMessage::InitTunnel(InitTunelRequest {
            endpoint_key: config.endpoint_key.clone(),
            admin_key: config.admin_key.clone(),
            proxies: vec![],
        }))
        .await?;

    match auth_response {
        InitTunnelResponse::Accepted { tunnel_id } => {
            info!("Tunnel accepted: {}", tunnel_id);
        }
        InitTunnelResponse::Rejected { reason } => {
            return Err(io::Error::new(io::ErrorKind::Other, reason));
        }
    }

    Ok(())
}
