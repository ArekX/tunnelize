use std::sync::Arc;

use log::{debug, info};
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::{
    common::{connection::Connection, data_bridge::DataBridge},
    server::{
        incoming_requests::InitLinkResponse,
        services::{ClientLink, LinkHandoff, Services},
        session::messages::ClientLinkResponse,
    },
};

pub async fn start(
    services: &Arc<Services>,
    client_id: Uuid,
    session_id: Uuid,
    mut response_stream: Connection,
    cancel_token: CancellationToken,
) {
    let Some(client_link) = services
        .get_client_manager()
        .await
        .take_client_link(&client_id)
    else {
        services.get_link_manager().await.respond_to_link(
            &session_id,
            ClientLinkResponse::Rejected {
                reason: "Client not found".to_string(),
            },
        );

        response_stream
            .respond_message(&InitLinkResponse::Rejected {
                reason: "Client not found".to_string(),
            })
            .await;
        return;
    };

    services
        .get_link_manager()
        .await
        .respond_to_link(&session_id, ClientLinkResponse::Accepted);

    response_stream
        .respond_message(&InitLinkResponse::Accepted)
        .await;

    match client_link {
        ClientLink::Stream {
            mut stream,
            initial_tunnel_data,
        } => {
            if let Some(data) = initial_tunnel_data
                && let Err(e) = response_stream.write_all(&data).await
            {
                debug!("Error writing initial tunnel data: {:?}", e);
                return;
            }

            tokio::select! {
                _ = cancel_token.cancelled() => {
                    debug!("Link session cancelled");
                }
                result = response_stream.bridge_to(&mut stream) => {
                    if let Err(e) = result {
                        debug!("Error linking session: {:?}", e);
                    }
                }
            }

            response_stream.shutdown().await;
            stream.shutdown().await;
        }
        ClientLink::Handoff(handoff_tx) => {
            let (release_tx, release_rx) = oneshot::channel();

            if let Err(mut handoff) = handoff_tx.send(LinkHandoff {
                stream: response_stream,
                release: release_tx,
            }) {
                debug!("Nobody is waiting for the link anymore, closing it");
                handoff.stream.shutdown().await;
                return;
            }

            // The receiver owns the stream now; stay registered until it drops
            // the release handle or the tunnel goes away.
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    debug!("Link session cancelled");
                }
                _ = release_rx => {}
            }
        }
    }

    info!("Link session, client_id '{}' ended", client_id);
}
