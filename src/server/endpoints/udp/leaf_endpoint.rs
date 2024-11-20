use std::sync::Arc;
use std::time::Duration;

use log::{debug, error, info, warn};
use tokio::io::Result;
use uuid::Uuid;

use crate::common::periodic_trigger::PeriodicTrigger;
use crate::common::udp_server::{ReceivedClient, UdpServer};

use crate::server::services::Client as MainClient;
use crate::server::session::messages::{ClientLinkRequest, ClientLinkResponse};

use super::udp_services::UdpServices;

pub async fn start(port: u16, services: Arc<UdpServices>) -> Result<()> {
    let config = services.get_config();
    let cancel_token = services.get_cancel_token();

    let (mut server, mut server_rx) = UdpServer::new(
        port,
        config.address.clone(),
        config.get_inactivity_timeout(),
        cancel_token.clone(),
    )
    .await?;

    let (trigger_handler, mut periodic_trigger) =
        PeriodicTrigger::new(Duration::from_secs(config.get_inactivity_timeout()));

    trigger_handler.start();

    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                trigger_handler.cancel();
                server.shutdown();
                server_rx.close();
                debug!("Leaf endpoint for UDP port '{}' cancelled.", port);
                break;
            }
            result = server.listen_for_connections() => {
                let Ok(client) = result else {
                    error!("Failed to accept connection.");
                    continue;
                };

                start_new_client(&services, client, port).await
            },
            data = server_rx.recv() => {
                match data {
                    Some(packet) => {
                        server.handle_channel_packet(packet).await;
                    }
                    None => {
                        cancel_token.cancel();
                    }
                }
            },
            _ = periodic_trigger.recv() => {
                server.cleanup_inactive_clients();
            }
        }
    }

    Ok(())
}

async fn start_new_client(services: &Arc<UdpServices>, received_client: ReceivedClient, port: u16) {
    let Some(tunnel) = services.get_tunnel_host().await.get_tunnel(port) else {
        error!(
            "No tunnel found for port {}. Stopping UDP connection.",
            port
        );
        return;
    };

    let client = MainClient::new(
        received_client.id,
        services.get_endpoint_name(),
        received_client.connection,
        Some(received_client.data),
    );

    let main_services = services.get_main_services();

    if let Err((error, link)) = main_services
        .get_client_manager()
        .await
        .subscribe_client(client)
    {
        if let Some(mut link) = link {
            link.stream.shutdown().await;
        }

        discard_client(received_client.id, &services).await;
        warn!("Failed to subscribe client: {}", error);
        return;
    }

    let Ok(response) = main_services
        .get_tunnel_manager()
        .await
        .send_session_request(
            &tunnel.tunnel_id,
            ClientLinkRequest {
                client_id: received_client.id,
                proxy_id: tunnel.proxy_id,
            },
        )
        .await
    else {
        error!("Error sending client link request");
        discard_client(received_client.id, &services).await;
        return;
    };

    match response {
        ClientLinkResponse::Accepted => {
            info!(
                "Client connected to tunnel {} on port {}",
                tunnel.tunnel_id, port
            );
        }
        ClientLinkResponse::Rejected { reason } => {
            error!("Client rejected by tunnel: {}", reason);
            discard_client(received_client.id, &services).await;
        }
    }
}

async fn discard_client(client_id: Uuid, services: &Arc<UdpServices>) {
    let main_services = services.get_main_services();

    if let Some(mut link) = main_services
        .get_client_manager()
        .await
        .take_client_link(&client_id)
    {
        link.stream.shutdown().await;
    }

    main_services
        .get_client_manager()
        .await
        .remove_client(&client_id);
}
