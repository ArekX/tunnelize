use std::sync::Arc;

use super::messages::{self, ClientLinkResponse, TunnelSessionRequest};
use log::{debug, info};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    common::{
        channel::{create_channel, Request, RequestReceiver, RequestSender},
        connection::ConnectionStream,
        transport::MessageError,
    },
    server::incoming_requests::ServerRequestMessage,
    tunnel::incoming_requests::{InitLinkRequest, InitLinkResponse, TunnelRequestMessage},
};

use super::super::services::Services;

#[derive(Clone, Debug)]
pub struct TunnelSession {
    id: Uuid,
    has_admin_privileges: bool,
    channel_tx: RequestSender<TunnelSessionRequest>,
}

impl TunnelSession {
    pub fn new(
        has_admin_privileges: bool,
        channel_tx: RequestSender<TunnelSessionRequest>,
    ) -> Self {
        let id = Uuid::new_v4();
        Self {
            id,
            has_admin_privileges,
            channel_tx,
        }
    }

    pub fn get_channel_tx(&self) -> RequestSender<TunnelSessionRequest> {
        self.channel_tx.clone()
    }

    pub fn get_id(&self) -> Uuid {
        self.id
    }
}

pub fn create(
    has_admin_privileges: bool,
) -> (TunnelSession, RequestReceiver<TunnelSessionRequest>) {
    let (channel_tx, channel_rx) = create_channel::<TunnelSessionRequest>();

    (
        TunnelSession::new(has_admin_privileges, channel_tx),
        channel_rx,
    )
}

pub async fn start(
    services: Arc<Services>,
    session: TunnelSession,
    mut stream: ConnectionStream,
    mut channel_rx: RequestReceiver<TunnelSessionRequest>,
) {
    let id = session.get_id();

    loop {
        tokio::select! {
            data = channel_rx.wait_for_requests() => {

                let Some(message) = data else {
                    break;
                };


                handle_channel_request(&services, &session, &mut stream, message).await;
            }
            message_result = stream.read_message::<ServerRequestMessage>() => {
                        match message_result {
                            Ok(ok_message) => {
                                handle_tunnel_message(&services, &session, ok_message).await;
                            }
                            Err(e) => match e {
                                MessageError::ConnectionClosed => {
                                    info!("Tunnel {} closed connection.", id);
                                    break;
                                }
                                _ => {
                                    debug!("Error while parsing {:?}", e);
                                    info!("Failed to read message from tunnel session {}: {}", id, e);
                                    continue;
                                }


                    }
                }
            }
        }
    }
}

pub async fn handle_channel_request(
    services: &Arc<Services>,
    session: &TunnelSession,
    stream: &mut ConnectionStream,
    mut request: Request<TunnelSessionRequest>,
) {
    match &mut request.data {
        TunnelSessionRequest::ClientLinkRequest(request_data) => {
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

pub async fn handle_tunnel_message(
    services: &Arc<Services>,
    session: &TunnelSession,
    message: ServerRequestMessage,
) {
    let id = session.get_id();

    println!("Handling message from tunnel session {}: {:?}", id, message);
    // TODO: Implement the rest of the tunnel session logic
}
