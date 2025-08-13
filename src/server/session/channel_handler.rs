use std::sync::Arc;

use log::{debug, info};
use uuid::Uuid;

use crate::{
    common::{
        channel::{Request, Responder},
        connection::Connection,
    },
    server::{
        services::{Services, events::ServiceEvent},
        session::messages::ClientLinkResponse,
    },
    tunnel::incoming_requests::{InitLinkRequest, InitLinkResponse},
};

use super::{
    messages::{ClientLinkRequest, TunnelChannelRequest},
    tunnel::TunnelSession,
};

pub async fn handle(
    services: &Arc<Services>,
    tunnel_session: &TunnelSession,
    stream: &mut Connection,
    mut request: Request<TunnelChannelRequest>,
) {
    let responder = request.take_responder();

    match request.data {
        TunnelChannelRequest::ClientLinkRequest(ref request_data) => {
            let info = {
                let Some(info) = services
                    .get_client_manager()
                    .await
                    .get_info(&request_data.client_id)
                else {
                    info!("Client {} not found", request_data.client_id);
                    return;
                };

                info
            };

            let link_session_id = {
                services.get_link_manager().await.create_link_session(
                    tunnel_session.get_id(),
                    info,
                    tunnel_session.get_child_cancel_token(),
                )
            };

            let response: InitLinkResponse = match stream
                .request_message(InitLinkRequest {
                    tunnel_id: tunnel_session.get_id(),
                    proxy_id: request_data.proxy_id,
                    session_id: link_session_id,
                })
                .await
            {
                Ok(response) => response,
                Err(e) => {
                    debug!("Failed to send init link response: {:?}", e);
                    reject_request(
                        services,
                        responder,
                        request_data,
                        link_session_id,
                        "Failed to initialize link session".to_string(),
                    )
                    .await;

                    return;
                }
            };

            match response {
                InitLinkResponse::Accepted => {
                    responder.respond(ClientLinkResponse::Accepted);
                }
                InitLinkResponse::Rejected { reason } => {
                    reject_request(services, responder, request_data, link_session_id, reason)
                        .await;
                }
            }
        }
    }
}

async fn reject_request(
    services: &Arc<Services>,
    responder: impl Responder<TunnelChannelRequest>,
    request_data: &ClientLinkRequest,
    link_session_id: Uuid,
    reason: String,
) {
    let link_rejected_event = ServiceEvent::LinkRejected {
        client_id: request_data.client_id,
        session_id: link_session_id,
    };

    responder.respond(ClientLinkResponse::Rejected { reason });

    services.push_event(link_rejected_event).await;
}
