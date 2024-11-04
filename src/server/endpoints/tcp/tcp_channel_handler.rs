use log::{debug, error, info};
use std::sync::Arc;
use tokio::io::Result;
use uuid::Uuid;

use crate::{
    common::channel::{OkResponse, Request},
    server::services::{Client, Services},
    server::session::messages::{ClientLinkRequest, ClientLinkResponse},
};

use super::{messages::TcpChannelRequest, tunnel_host::TunnelHost};

pub async fn handle(
    mut request: Request<TcpChannelRequest>,
    name: &str,
    tunnel_host: &mut TunnelHost,
    services: &Arc<Services>,
) -> Result<()> {
    match &mut request.data {
        TcpChannelRequest::ClientConnect(client_request) => {
            // TODO: Simplify this, move into leaf_endpoint
            let Some(tunnel) = tunnel_host.get_tunnel(client_request.port) else {
                error!("No tunnel found for port {}", client_request.port);

                if let Some(mut stream) = client_request.stream.take() {
                    stream.shutdown().await;
                }

                request.respond(OkResponse);
                return Ok(());
            };

            debug!(
                "Found tunnel for port {}: {}",
                client_request.port, tunnel.tunnel_id
            );

            let Some(stream) = client_request.stream.take() else {
                error!("Client stream is missing");
                return Ok(());
            };
            let client_id = Uuid::new_v4();
            let client = Client::new(client_id, name.to_owned(), stream, None, None);

            services.get_client_manager().await.subscribe_client(client);

            let Ok(response) = services
                .get_tunnel_manager()
                .await
                .send_session_request(
                    &tunnel.tunnel_id,
                    ClientLinkRequest {
                        client_id,
                        proxy_id: tunnel.proxy_id,
                    },
                )
                .await
            else {
                error!("Error sending client link request");
                discard_client(client_id, services).await;
                return Ok(());
            };

            match response {
                ClientLinkResponse::Accepted => {
                    info!(
                        "Client connected to tunnel {} on port {}",
                        tunnel.tunnel_id, client_request.port
                    );
                }
                ClientLinkResponse::Rejected { reason } => {
                    error!("Client rejected by tunnel: {}", reason);
                    discard_client(client_id, services).await;
                }
            }

            request.respond(OkResponse);
        }
    }

    Ok(())
}

async fn discard_client(client_id: Uuid, services: &Arc<Services>) {
    if let Some(mut link) = services
        .get_client_manager()
        .await
        .take_client_link(&client_id)
    {
        link.stream.shutdown().await;
    }

    services
        .get_client_manager()
        .await
        .remove_client(&client_id);
}
