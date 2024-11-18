use std::sync::Arc;

use log::warn;
use log::{debug, error, info};
use tokio::io::{Error, ErrorKind, Result};
use uuid::Uuid;

use crate::common::connection::Connection;
use crate::common::tcp_server::TcpServer;
use crate::server::services::Client;
use crate::server::services::Services as MainServices;
use crate::server::session::messages::{ClientLinkRequest, ClientLinkResponse};

use super::tcp_services::TcpServices;

pub async fn start(port: u16, services: Arc<TcpServices>) -> Result<()> {
    let cancel_token = services.get_cancel_token();
    let config = services.get_config();

    let listener =
        match TcpServer::new(config.get_address(), port, services.get_server_encryption()).await {
            Ok(listener) => listener,
            Err(e) => {
                error!("Failed to bind client listener: {}", e);
                return Err(Error::new(
                    ErrorKind::Other,
                    "Failed to bind client listener",
                ));
            }
        };

    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                debug!("Leaf endpoint for TCP port '{}' cancelled.", port);
                break;
            }
            result = listener.listen_for_connection() => {
                 let Ok((connection, address)) = result else {
                    error!("Failed to accept connection.");
                    continue;
                };

                debug!(
                    "Accepted TCP connection from client '{}' at port {}",
                    address, port
                );

                start_client(port, connection, &services).await;
            }
        }
    }

    Ok(())
}

pub async fn start_client(port: u16, mut connection: Connection, services: &Arc<TcpServices>) {
    let tunnel_host = services.get_tunnel_host().await;

    let Some(tunnel) = tunnel_host.get_tunnel(port) else {
        error!("No tunnel found for port {}", port);
        connection.shutdown().await;
        return;
    };

    debug!("Found tunnel for port {}: {}", port, tunnel.tunnel_id);

    let client_id = Uuid::new_v4();
    let client = Client::new(client_id, services.get_endpoint_name(), connection, None);

    let main_services = services.get_main_services();
    if let Err((error, link)) = main_services
        .get_client_manager()
        .await
        .subscribe_client(client)
    {
        if let Some(mut link) = link {
            link.stream.shutdown().await;
        }

        discard_client(client_id, &main_services).await;
        warn!("Failed to subscribe client: {}", error);
        return;
    }

    let Ok(response) = main_services
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
        discard_client(client_id, &main_services).await;
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
            discard_client(client_id, &main_services).await;
        }
    }
}

async fn discard_client(client_id: Uuid, services: &Arc<MainServices>) {
    let mut client_manager = services.get_client_manager().await;

    if let Some(mut link) = client_manager.take_client_link(&client_id) {
        link.stream.shutdown().await;
    }

    client_manager.remove_client(&client_id);
}
