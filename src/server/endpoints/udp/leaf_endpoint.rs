use std::sync::Arc;
use std::time::Duration;

use log::{debug, error, info, warn};
use tokio::io::Result;
use tokio::sync::mpsc::channel;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::common::channel_socket::ChannelPacket;
use crate::common::periodic_trigger::PeriodicTrigger;
use crate::common::udp_server::{Client, UdpServer};
use crate::server::endpoints::udp::client_host::Host;

use crate::server::services::Client as MainClient;
use crate::server::session::messages::{ClientLinkRequest, ClientLinkResponse};

use super::udp_services::UdpServices;

pub async fn start(port: u16, services: Arc<UdpServices>) -> Result<()> {
    let config = services.get_config();
    let cancel_token = services.get_cancel_token();

    let (leaf_data_tx, mut leaf_data_rx) = channel::<ChannelPacket>(100);

    let mut server = UdpServer::new(
        port,
        config.address.clone(),
        leaf_data_tx.clone(),
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
                debug!("Leaf endpoint for UDP port '{}' cancelled.", port);
                break;
            }
            result = server.listen_for_connections() => {
                let Ok(client) = result else {
                    error!("Failed to accept connection.");
                    continue;
                };

                start_new_client(&services, client, port, cancel_token.child_token()).await
            },
            data = leaf_data_rx.recv() => {
                match data {
                    Some(ChannelPacket(client_id, data)) => {
                        println!("Sending data to client: {}", client_id);

                        let client_host = services.get_client_host().await;

                        if let Some(client_address) = client_host.get_client_address(&client_id) {
                            if let Err(e) = server.write(&data, &client_address).await {
                                debug!("Failed to send data to client. Reason: {}", e);
                                continue;
                            };
                        } else {
                            error!("No target address set, cannot send UDP datagram at port '{}'.", port);
                        }
                    }
                    None => {
                        cancel_token.cancel();
                        continue;
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

async fn start_new_client(
    services: &Arc<UdpServices>,
    udp_client: Client,
    port: u16,
    cancel_child_token: CancellationToken,
) {
    let Some(tunnel) = services.get_tunnel_host().await.get_tunnel(port) else {
        error!(
            "No tunnel found for port {}. Stopping UDP connection.",
            port
        );
        return;
    };

    services.get_client_host().await.add(Host::new(
        udp_client.id,
        udp_client.address,
        tunnel.tunnel_id,
        cancel_child_token.clone(),
    ));

    let client = MainClient::new(
        udp_client.id,
        services.get_endpoint_name(),
        udp_client.connection,
        Some(udp_client.data),
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

        discard_client(udp_client.id, &services).await;
        warn!("Failed to subscribe client: {}", error);
        return;
    }

    let Ok(response) = main_services
        .get_tunnel_manager()
        .await
        .send_session_request(
            &tunnel.tunnel_id,
            ClientLinkRequest {
                client_id: udp_client.id,
                proxy_id: tunnel.proxy_id,
            },
        )
        .await
    else {
        error!("Error sending client link request");
        discard_client(udp_client.id, &services).await;
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
            discard_client(udp_client.id, &services).await;
        }
    }
}

async fn discard_client(client_id: Uuid, services: &Arc<UdpServices>) {
    let main_services = services.get_main_services();

    services.get_client_host().await.cancel_client(&client_id);

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
