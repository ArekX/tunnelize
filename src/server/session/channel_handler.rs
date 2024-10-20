use std::sync::Arc;

use log::info;

use crate::{
    common::{channel::Request, connection::ConnectionStream},
    server::{services::Services, session::messages::ClientLinkResponse},
    tunnel::incoming_requests::{InitLinkRequest, InitLinkResponse},
};

use super::{messages::TunnelChannelRequest, tunnel::TunnelSession};

pub async fn handle(
    services: &Arc<Services>,
    session: &TunnelSession,
    stream: &mut ConnectionStream,
    mut request: Request<TunnelChannelRequest>,
) {
    match &mut request.data {
        TunnelChannelRequest::ClientLinkRequest(request_data) => {
            let link_session_id = services
                .get_link_manager()
                .await
                .create_link_session(session.get_id(), request_data.client_id);

            info!(
                "Created link session {} for client {}",
                link_session_id, request_data.client_id
            );

            let response: InitLinkResponse = match stream
                .request_message(InitLinkRequest {
                    tunnel_id: session.get_id(),
                    proxy_id: request_data.proxy_id,
                    session_id: link_session_id,
                })
                .await
            {
                Ok(response) => response,
                Err(e) => {
                    info!("Failed to send InitLinkSession request: {}", e);
                    return;
                }
            };

            match response {
                InitLinkResponse::Accepted => {
                    request.respond(ClientLinkResponse::Accepted).await;
                }
                InitLinkResponse::Rejected { reason } => {
                    request
                        .respond(ClientLinkResponse::Rejected { reason })
                        .await;
                }
            }
        }
    }
}
