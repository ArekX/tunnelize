use std::sync::Arc;

use log::{debug, info};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::{
    common::{connection::Connection, data_bridge::DataBridge},
    server::{incoming_requests::InitLinkResponse, services::Services},
};

pub async fn start(
    services: &Arc<Services>,
    client_id: Uuid,
    mut response_stream: Connection,
    cancel_token: CancellationToken,
) {
    let mut client_link = {
        let Some(client_link) = services
            .get_client_manager()
            .await
            .take_client_link(&client_id)
        else {
            response_stream
                .respond_message(&InitLinkResponse::Rejected {
                    reason: "Client not found".to_string(),
                })
                .await;
            return;
        };

        client_link
    };

    response_stream
        .respond_message(&InitLinkResponse::Accepted)
        .await;

    if let Some(data) = client_link.initial_tunnel_data {
        if let Err(e) = response_stream.write_all(&data).await {
            debug!("Error writing initial tunnel data: {:?}", e);
            return;
        }
    }

    tokio::select! {
        _ = cancel_token.cancelled() => {
            debug!("Link session cancelled");
        }
        result = response_stream.bridge_to(&mut client_link.stream, client_link.context) => {
            if let Err(e) = result {
                debug!("Error linking session: {:?}", e);
            }
        }
    }

    response_stream.shutdown().await;
    client_link.stream.shutdown().await;

    info!("Link session, client_id '{}' ended", client_id);
}
