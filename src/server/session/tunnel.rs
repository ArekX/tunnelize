use std::{ops::ControlFlow, sync::Arc};

use super::messages::TunnelChannelRequest;
use log::info;
use serde::Serialize;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::{
    common::{
        channel::{create_channel, RequestReceiver, RequestSender},
        connection::ConnectionStream,
    },
    server::{
        endpoints::messages::ResolvedEndpointInfo, services::TunnelInfo, session::channel_handler,
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

#[derive(Debug, Serialize, Clone)]
pub struct TunnelProxyInfo {
    pub endpoint: String,
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
    mut stream: ConnectionStream,
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
            Ok(ControlFlow::Break(())) = stream.wait_for_data() => {
                info!("Tunnel {} session has been closed.", id);
                session.cancel();
                break;
            }
        }
    }
}
