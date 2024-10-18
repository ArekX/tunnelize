use std::sync::Arc;

use log::{debug, info};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{common::data_request::DataRequest, connect_data_response};

use super::super::services::Services;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InitLinkRequest {
    pub tunnel_id: Uuid,
    pub session_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum InitLinkResponse {
    Accepted,
    Rejected { reason: String },
}

connect_data_response!(InitLinkRequest -> InitLinkResponse);

pub async fn process_init_link(services: Arc<Services>, mut request: DataRequest<InitLinkRequest>) {
    let Some(client_id) = services
        .get_link_manager()
        .await
        .resolve_tunnel_session_client(&request.data.session_id, &request.data.tunnel_id)
    else {
        request
            .response_stream
            .respond_message(&InitLinkResponse::Rejected {
                reason: "Session not found".to_string(),
            })
            .await;
        return;
    };

    let Some(mut client_link) = services
        .get_client_manager()
        .await
        .take_client_link(&client_id)
    else {
        request
            .response_stream
            .respond_message(&InitLinkResponse::Rejected {
                reason: "Client not found".to_string(),
            })
            .await;
        return;
    };

    request
        .response_stream
        .respond_message(&InitLinkResponse::Accepted)
        .await;

    if let Some(data) = client_link.initial_tunnel_data {
        if let Err(e) = request.response_stream.write_all(&data).await {
            debug!("Error writing initial tunnel data: {:?}", e);
            return;
        }
    }

    if let Err(e) = request
        .response_stream
        .link_session_with(&mut client_link.stream)
        .await
    {
        debug!("Error linking session: {:?}", e);
    }

    services
        .get_client_manager()
        .await
        .remove_client(&client_id);

    services
        .get_link_manager()
        .await
        .remove_session(&request.data.session_id);
}
