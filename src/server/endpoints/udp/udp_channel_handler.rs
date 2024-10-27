use log::{debug, error, info};
use std::sync::Arc;
use tokio::io::Result;
use uuid::Uuid;

use crate::{
    common::{
        channel::{OkResponse, Request},
        connection::ConnectionStreamContext,
    },
    server::{
        services::{Client, Services},
        session::messages::{ClientLinkRequest, ClientLinkResponse},
    },
};

use super::{messages::UdpChannelRequest, tunnel_host::TunnelHost};

pub async fn handle(
    mut request: Request<UdpChannelRequest>,
    name: &str,
    tunnel_host: &mut TunnelHost,
    services: &Arc<Services>,
) -> Result<()> {
    match &mut request.data {
        UdpChannelRequest::ClientConnect(client_request) => {
            let Some(tunnel) = tunnel_host.get_tunnel(client_request.port) else {
                error!("No tunnel found for port {}", client_request.port);
                request.respond(OkResponse).await;
                return Ok(());
            };

            debug!(
                "Found tunnel for port {}: {}",
                client_request.port, tunnel.tunnel_id
            );

            let client_id = Uuid::new_v4();

            let Some(stream) = client_request.stream.take() else {
                error!("No stream found for client.");
                return Ok(());
            };

            let Some(session) = client_request.session.take() else {
                error!("No session found for client.");
                return Ok(());
            };

            let client = Client::new(
                client_id,
                name.to_owned(),
                stream,
                Some(ConnectionStreamContext::Udp(session)),
                client_request.initial_data.take(),
            );

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

            request.respond(OkResponse).await;
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