use std::sync::Arc;

use super::messages::{self, ClientLinkResponse, TunnelSessionMessage};
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
    channel_tx: RequestSender<TunnelSessionMessage>,
}

impl TunnelSession {
    pub fn new(
        has_admin_privileges: bool,
        channel_tx: RequestSender<TunnelSessionMessage>,
    ) -> Self {
        let id = Uuid::new_v4();
        Self {
            id,
            has_admin_privileges,
            channel_tx,
        }
    }

    pub fn get_channel_tx(&self) -> RequestSender<TunnelSessionMessage> {
        self.channel_tx.clone()
    }

    pub fn get_id(&self) -> Uuid {
        self.id
    }
}

pub fn create(
    has_admin_privileges: bool,
) -> (TunnelSession, RequestReceiver<TunnelSessionMessage>) {
    let (channel_tx, channel_rx) = create_channel::<TunnelSessionMessage>();

    (
        TunnelSession::new(has_admin_privileges, channel_tx),
        channel_rx,
    )
}

pub async fn start(
    services: Arc<Services>,
    session: TunnelSession,
    mut stream: ConnectionStream,
    mut channel_rx: RequestReceiver<TunnelSessionMessage>,
) {
    let id = session.get_id();

    loop {
        tokio::select! {
            data = channel_rx.recv() => {

                let Some(message) = data else {
                    break;
                };


                handle_channel_message(&services, &session, &mut stream, message).await;
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

pub async fn handle_channel_message(
    services: &Arc<Services>,
    session: &TunnelSession,
    stream: &mut ConnectionStream,
    mut request: Request<TunnelSessionMessage>,
) {
    match &mut request.data {
        TunnelSessionMessage::EndpointInfo(info) => {
            println!("Endpoint info: {:?}", info);
        }
        TunnelSessionMessage::ClientLinkRequest(ref mut requestx) => {
            let response: InitLinkResponse = match stream
                .request_message(&TunnelRequestMessage::InitLinkSession(InitLinkRequest {
                    tunnel_id: session.get_id(),
                    proxy_id: Uuid::new_v4(), // FIXME: Proxy ID should be generated on server
                    session_id: Uuid::new_v4(), // FIXME: Store session id with link to client_id
                }))
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
                    request.respond(ClientLinkResponse::Accepted);
                }
                InitLinkResponse::Rejected { reason } => {
                    request.respond(ClientLinkResponse::Rejected { reason });
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
