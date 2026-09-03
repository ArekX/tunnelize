use std::sync::Arc;

use log::{debug, info};
use uuid::Uuid;

use crate::{
    common::{channel::Request, connection::Connection},
    server::{
        services::{Services, events::ServiceEvent},
        session::messages::ClientLinkResponse,
    },
    tunnel::incoming_requests::{InitLinkRequest, TunnelRequestMessage},
};

use super::{messages::TunnelChannelRequest, tunnel::TunnelSession};

pub async fn handle(
    services: &Arc<Services>,
    tunnel_session: &TunnelSession,
    stream: &mut Connection,
    mut request: Request<TunnelChannelRequest>,
) {
    let responder = request.take_response_tx();

    match request.data {
        TunnelChannelRequest::ClientLinkRequest(ref request_data) => {
            let Some(info) = services
                .get_client_manager()
                .await
                .get_info(&request_data.client_id)
            else {
                info!("Client {} not found", request_data.client_id);
                if let Some(responder) = responder {
                    let _ = responder.send(
                        ClientLinkResponse::Rejected {
                            reason: "Client not found".to_string(),
                        }
                        .into(),
                    );
                }
                return;
            };

            let link_session_id = services.get_link_manager().await.create_link_session(
                tunnel_session.get_id(),
                info,
                tunnel_session.get_child_cancel_token(),
                responder,
            );

            let message = TunnelRequestMessage::InitLinkRequest(InitLinkRequest {
                tunnel_id: tunnel_session.get_id(),
                proxy_id: request_data.proxy_id,
                session_id: link_session_id,
            });

            if let Err(e) = stream.write_message(&message).await {
                debug!("Failed to send init link request: {:?}", e);
                reject_link(
                    services,
                    request_data.client_id,
                    link_session_id,
                    "Failed to initialize link session".to_string(),
                )
                .await;
            }
        }
    }
}

pub async fn reject_link(
    services: &Arc<Services>,
    client_id: Uuid,
    link_session_id: Uuid,
    reason: String,
) {
    services
        .get_link_manager()
        .await
        .respond_to_link(&link_session_id, ClientLinkResponse::Rejected { reason });

    services
        .push_event(ServiceEvent::LinkRejected {
            client_id,
            session_id: link_session_id,
        })
        .await;
}
