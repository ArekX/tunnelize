use std::{io::ErrorKind, sync::Arc};

use log::error;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

use crate::{
    server::session::messages::{LinkRejected, TunnelSessionMessage},
    tunnel::{outgoing_requests, services::Services},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InitLinkRequest {
    pub tunnel_id: Uuid,
    pub proxy_id: Uuid,
    pub session_id: Uuid,
}

pub async fn process_init_link(
    services: Arc<Services>,
    request: InitLinkRequest,
    control_tx: Sender<TunnelSessionMessage>,
) {
    let Some((address, port)) = services
        .get_proxy_manager()
        .await
        .get_forward_address(&request.proxy_id)
    else {
        reject_link(
            &control_tx,
            request.session_id,
            "Requested proxy not found".to_string(),
        )
        .await;
        return;
    };

    if let Err(e) =
        outgoing_requests::start_link_session(services, request.proxy_id, request.session_id).await
    {
        error!("Failed to start link session: {}", e);

        let message = if let ErrorKind::ConnectionRefused = e.kind() {
            format!("Connection refused, could not connect to source at {address}:{port}")
        } else {
            format!("Failed to start link session: {}", e.kind())
        };

        reject_link(&control_tx, request.session_id, message).await;
    }
}

async fn reject_link(control_tx: &Sender<TunnelSessionMessage>, session_id: Uuid, reason: String) {
    if control_tx
        .send(TunnelSessionMessage::LinkRejected(LinkRejected {
            session_id,
            reason,
        }))
        .await
        .is_err()
    {
        error!("Failed to report rejected link, control connection is gone");
    }
}
