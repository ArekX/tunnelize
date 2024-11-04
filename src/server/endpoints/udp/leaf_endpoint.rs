use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

use bytes::BytesMut;
use log::{debug, error, info};
use tokio::io::{Error, ErrorKind, Result};
use tokio::net::UdpSocket;
use tokio::sync::mpsc::channel;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::common::channel_socket::{ChannelPacket, ChannelSocket};
use crate::common::connection::{Connection, ConnectionStreamContext};
use crate::common::data_bridge::UdpSession;
use crate::server::endpoints::udp::client_host::Client;

use crate::server::services::Client as MainClient;
use crate::server::session::messages::{ClientLinkRequest, ClientLinkResponse};

use super::udp_services::UdpServices;

pub async fn start(port: u16, services: Arc<UdpServices>) -> Result<()> {
    let config = services.get_config();
    let cancel_token = services.get_cancel_token();

    let mut connection = match UdpSocket::bind(config.get_bind_address(port)).await {
        Ok(socket) => Connection::from(socket),
        Err(e) => {
            error!("Failed to bind client listener: {}", e);
            return Err(Error::new(
                ErrorKind::Other,
                "Failed to bind client listener",
            ));
        }
    };

    let (leaf_data_tx, mut leaf_data_rx) = channel::<ChannelPacket>(100);

    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                debug!("Leaf endpoint for UDP port '{}' cancelled.", port);
                break;
            }
            result = wait_for_client(&mut connection) => {
                let Ok((data, address)) = result else {
                    error!("Failed to accept connection.");
                    continue;
                };

                {
                    let mut client_host = services.get_client_host().await;
                    if client_host.client_exists(&address) {
                        client_host.send(&address, data).await;
                        continue;
                    }
                }

                start_new_client(&services, data, address, port, cancel_token.child_token(), leaf_data_tx.clone()).await
            },
            data = leaf_data_rx.recv() => {
                match data {
                    Some(ChannelPacket(client_id, data)) => {
                        if let Some(client_address) = services.get_client_host().await.get_client_address(&client_id) {
                            if let Err(e) = connection.write_all_to(&data, &client_address).await {
                                debug!("Failed to send data to client. Reason: {}", e);
                                continue;
                            };
                            services.get_client_host().await.update_activity(&client_id);
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
        }
    }

    Ok(())
}

async fn wait_for_client(connection: &mut Connection) -> Result<(Vec<u8>, SocketAddr)> {
    let mut buffer = BytesMut::with_capacity(2048);
    buffer.resize(2048, 0);

    match connection.read_with_address(&mut buffer).await {
        Ok((size, address)) => Ok((buffer[..size].to_vec(), address)),
        Err(e) => {
            error!("Failed to read data from client: {:?}", e);
            Err(e)
        }
    }
}

async fn start_new_client(
    services: &Arc<UdpServices>,
    initial_data: Vec<u8>,
    address: SocketAddr,
    port: u16,
    cancel_child_token: CancellationToken,
    leaf_data_tx: Sender<ChannelPacket>,
) {
    let Some(tunnel) = services.get_tunnel_host().await.get_tunnel(port) else {
        error!(
            "No tunnel found for port {}. Stopping UDP connection.",
            port
        );
        return;
    };

    let channel_socket = ChannelSocket::new(leaf_data_tx.clone(), cancel_child_token.clone());

    let client_id = channel_socket.get_id();

    services.get_client_host().await.add(Client::new(
        client_id,
        address,
        tunnel.tunnel_id,
        channel_socket.get_socket_tx(),
        cancel_child_token.clone(),
    ));

    let client = MainClient::new(
        client_id,
        services.get_endpoint_name(),
        channel_socket.into(),
        Some(ConnectionStreamContext::Udp(UdpSession {
            address,
            cancel_token: cancel_child_token,
        })),
        Some(initial_data),
    );

    let main_services = services.get_main_services();

    main_services
        .get_client_manager()
        .await
        .subscribe_client(client);

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
        discard_client(client_id, &services).await;
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
            discard_client(client_id, &services).await;
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
