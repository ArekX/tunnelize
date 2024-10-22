use std::sync::Arc;

use super::messages::TunnelChannelRequest;
use log::{debug, info};
use serde::Serialize;
use uuid::Uuid;

use crate::{
    common::{
        channel::{create_channel, RequestReceiver, RequestSender},
        connection::ConnectionStream,
        transport::MessageError,
    },
    server::{
        endpoints::messages::EndpointInfo, incoming_requests::ServerRequestMessage,
        services::TunnelInfo, session::channel_handler,
    },
};

use super::super::services::Services;

#[derive(Clone, Debug)]
pub struct TunnelSession {
    id: Uuid,
    name: Option<String>,
    proxies: Vec<TunnelProxyInfo>,
    has_admin_privileges: bool,
    channel_tx: RequestSender<TunnelChannelRequest>,
}

impl TunnelSession {
    pub fn new(
        id: Uuid,
        name: Option<String>,
        proxies: Vec<TunnelProxyInfo>,
        has_admin_privileges: bool,
        channel_tx: RequestSender<TunnelChannelRequest>,
    ) -> Self {
        Self {
            id,
            name,
            proxies,
            has_admin_privileges,
            channel_tx,
        }
    }

    pub fn get_channel_tx(&self) -> RequestSender<TunnelChannelRequest> {
        self.channel_tx.clone()
    }

    pub fn get_id(&self) -> Uuid {
        self.id
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct TunnelProxyInfo {
    pub endpoint: String,
    pub details: EndpointInfo,
}

impl Into<TunnelInfo> for &TunnelSession {
    fn into(self) -> TunnelInfo {
        TunnelInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            proxies: self.proxies.clone(),
            has_admin_access: self.has_admin_privileges.clone(),
        }
    }
}

pub fn create(
    id: Uuid,
    name: Option<String>,
    proxies: Vec<TunnelProxyInfo>,
    has_admin_privileges: bool,
) -> (TunnelSession, RequestReceiver<TunnelChannelRequest>) {
    let (channel_tx, channel_rx) = create_channel::<TunnelChannelRequest>();

    (
        TunnelSession::new(id, name, proxies, has_admin_privileges, channel_tx),
        channel_rx,
    )
}

pub async fn start(
    services: Arc<Services>,
    session: TunnelSession,
    mut stream: ConnectionStream,
    mut channel_rx: RequestReceiver<TunnelChannelRequest>,
) {
    let id = session.get_id();

    loop {
        tokio::select! {
            data = channel_rx.wait_for_requests() => {

                let Some(message) = data else {
                    break;
                };


                channel_handler::handle(&services, &session, &mut stream, message).await;
            }
            message_result = stream.read_message::<ServerRequestMessage>() => {
                match message_result {
                    Ok(ok_message) => {
                        // TODO: Remove if this not needed in the end.
                        println!("Received message from tunnel session {}: {:?}", id, ok_message);
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
