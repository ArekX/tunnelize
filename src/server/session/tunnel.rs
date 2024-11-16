use std::sync::Arc;

use super::messages::TunnelChannelRequest;
use log::{debug, info};
use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::{
    common::{
        channel::{create_channel, RequestReceiver, RequestSender},
        connection::Connection,
        transport::MessageError,
    },
    server::{
        endpoints::messages::ResolvedEndpointInfo, incoming_requests::ServerRequestMessage,
        services::TunnelInfo, session::channel_handler,
    },
};

use super::super::services::Services;

#[derive(Clone, Debug)]
pub struct TunnelSession {
    id: Uuid,
    name: Option<String>,
    proxies: Vec<TunnelProxyInfo>,
    channel_tx: RequestSender<TunnelChannelRequest>,
    cancel_token: CancellationToken,
}

impl TunnelSession {
    pub fn new(
        id: Uuid,
        name: Option<String>,
        proxies: Vec<TunnelProxyInfo>,
        channel_tx: RequestSender<TunnelChannelRequest>,
    ) -> Self {
        Self {
            id,
            name,
            proxies,
            channel_tx,
            cancel_token: CancellationToken::new(),
        }
    }

    pub fn get_cancel_token(&self) -> CancellationToken {
        self.cancel_token.clone()
    }

    pub fn get_child_cancel_token(&self) -> CancellationToken {
        self.cancel_token.child_token()
    }

    pub fn cancel(&self) {
        self.cancel_token.cancel();
    }

    pub async fn wait_for_cancellation(&self) {
        self.cancel_token.cancelled().await;
    }

    pub fn get_channel_tx(&self) -> RequestSender<TunnelChannelRequest> {
        self.channel_tx.clone()
    }

    pub fn get_id(&self) -> Uuid {
        self.id
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TunnelProxyInfo {
    pub endpoint: String,
    pub forward_address: String,
    pub forward_port: u16,
    pub details: ResolvedEndpointInfo,
}

impl Into<TunnelInfo> for &TunnelSession {
    fn into(self) -> TunnelInfo {
        TunnelInfo {
            id: self.id.clone(),
            name: self.name.clone(),
            proxies: self.proxies.clone(),
        }
    }
}

pub fn create(
    id: Uuid,
    name: Option<String>,
    proxies: Vec<TunnelProxyInfo>,
) -> (TunnelSession, RequestReceiver<TunnelChannelRequest>) {
    let (channel_tx, channel_rx) = create_channel::<TunnelChannelRequest>();

    (
        TunnelSession::new(id, name, proxies, channel_tx),
        channel_rx,
    )
}

pub async fn start(
    services: Arc<Services>,
    session: TunnelSession,
    mut stream: Connection,
    mut channel_rx: RequestReceiver<TunnelChannelRequest>,
) {
    let id = session.get_id();

    loop {
        tokio::select! {
            _ = session.wait_for_cancellation() => {
                info!("Tunnel {} session has been cancelled.", id);
                break;
            }
            data = channel_rx.wait_for_requests() => {

                let Some(message) = data else {
                    session.cancel();
                    break;
                };


                channel_handler::handle(&services, &session, &mut stream, message).await;
            },
            result = stream.read_message::<ServerRequestMessage>() => {
                match result {
                    Ok(message) => {
                        debug!("Received unexpected message from client: {:?}", message);
                    },
                    Err(MessageError::ConnectionClosed) => {
                        debug!("Tunnel Connection closed.");
                        session.cancel();
                    },
                    Err(e) => {
                        info!("Failed to read message from client: {}", e);
                        session.cancel();
                        break;
                    }
                }
            }
        }
    }

    stream.shutdown().await;
    channel_rx.close();
}
