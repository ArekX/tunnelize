use std::{io::ErrorKind, sync::Arc};

use log::error;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    common::connection::Connection,
    tunnel::{outgoing_requests, services::Services},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InitLinkRequest {
    pub tunnel_id: Uuid,
    pub proxy_id: Uuid,
    pub session_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum InitLinkResponse {
    Accepted,
    Rejected { reason: String },
}

pub async fn process_init_link(
    services: &Arc<Services>,
    request: InitLinkRequest,
    response_stream: &mut Connection,
) {
    let Some((address, port)) = services
        .get_proxy_manager()
        .await
        .get_forward_address(&request.proxy_id)
    else {
        response_stream
            .respond_message(&InitLinkResponse::Rejected {
                reason: "Requested proxy not found".to_string(),
            })
            .await;
        return;
    };

    let address_port = format!("{}:{}", address, port);

    {
        if let Err(e) = outgoing_requests::start_link_session(
            services.clone(),
            request.proxy_id,
            request.session_id,
        )
        .await
        {
            error!("Failed to start link session: {}", e);

            let message = if let ErrorKind::ConnectionRefused = e.kind() {
                format!(
                    "Connection refused, could not connect to source at {}",
                    address_port
                )
            } else {
                format!("Failed to start link session: {}", e.kind())
            };

            response_stream
                .respond_message(&InitLinkResponse::Rejected { reason: message })
                .await;
            return;
        }

        response_stream
            .respond_message(&InitLinkResponse::Accepted)
            .await;
    };
}
