use std::time::Duration;

use log::{debug, error, info, warn};
use tokio::{
    io::{self, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    time::timeout,
};
use uuid::Uuid;

use crate::{
    http::{client_resolver::resolve_http_client, messages::ServerMessage},
    transport::{write_message, MessageError},
};

use super::{
    client_list::ClientList, host_list::HostList, tunnel_list::TunnelList, HttpServerConfig,
    TaskData, TaskService,
};

async fn respond_and_close(stream: &mut TcpStream, message: &str) {
    if let Err(e) = stream.write_all(message.as_bytes()).await {
        error!("Failed to respond to client: {}", e);
    }

    if let Err(e) = stream.shutdown().await {
        error!("Failed to close client connection: {}", e);
    }
}

async fn wait_for_client_readable(stream: &mut TcpStream) -> bool {
    let duration = Duration::from_secs(5);
    match timeout(duration, stream.readable()).await {
        Ok(_) => true,
        Err(_) => {
            debug!("Timeout while waiting for client stream to be readable.");
            false
        }
    }
}

pub async fn start_http_server(
    config: TaskData<HttpServerConfig>,
    host_service: TaskService<HostList>,
    tunnel_service: TaskService<TunnelList>,
    client_service: TaskService<ClientList>,
) {
    let client = match TcpListener::bind(format!("0.0.0.0:{}", config.client_port)).await {
        Ok(listener) => listener,
        Err(e) => {
            error!("Failed to bind client listener: {}", e);
            return;
        }
    };

    info!(
        "Listening to client connections on 0.0.0.0:{}",
        config.client_port
    );

    loop {
        let (mut stream, address) = match client.accept().await {
            Ok(stream_pair) => stream_pair,
            Err(e) => {
                error!("Failed to accept client connection: {}", e);
                continue;
            }
        };

        info!("Client connected from {}", address);

        info!("Waiting for client to be readable...");
        if !wait_for_client_readable(&mut stream).await {
            warn!("Client stream not readable or pre-connection without sending data, closing connection.");
            if let Err(e) = stream.shutdown().await {
                debug!("Error while closing client stream. {:?}", e);
            }
            continue;
        }

        let resolved_client = resolve_http_client(&mut stream).await;

        if let None = resolved_client.resolved_host {
            info!("No hostname found in initial request, closing connection.");
            respond_and_close(
                &mut stream,
                "No hostname found for this request. Cannot resolve to a tunnel. Closing connection.",
            ).await;
            continue;
        }

        info!(
            "Resolved hostname {} from initial request",
            resolved_client.resolved_host.clone().unwrap()
        );
        let host = {
            let host_service = host_service.lock().await;
            match host_service.find_host(&resolved_client.resolved_host.clone().unwrap()) {
                Some(host) => host,
                None => {
                    error!(
                        "Failed to find host for hostname {}",
                        resolved_client.resolved_host.clone().unwrap()
                    );
                    respond_and_close(
                        &mut stream,
                        "No tunnel connected for this hostname. Closing connection.",
                    )
                    .await;
                    continue;
                }
            }
        };

        let client_id = {
            let mut client_service = client_service.lock().await;
            client_service.issue_client_id()
        };

        {
            info!(
                "Client connected from {}, assigned ID: {}",
                address, client_id
            );
            client_service.lock().await.register(
                client_id,
                stream,
                resolved_client.initial_request,
            );
        }

        let mut tunnel_service = tunnel_service.lock().await;

        let tunnel = match tunnel_service.get_by_id(host.tunnel_id) {
            Some(tunnel) => tunnel,
            None => {
                error!("Failed to find tunnel for ID: {}", host.tunnel_id);
                end_client(
                    &client_service,
                    client_id,
                    "No tunnel connected for this hostname. Closing connection.",
                )
                .await;
                continue;
            }
        };

        debug!(
            "Sending link request to tunnel for client ID {}, host ID: {} -> tunnel ID: {}",
            client_id, host.host_id, host.tunnel_id
        );
        match write_message(
            &mut tunnel.stream,
            &ServerMessage::ClientLinkRequest {
                client_id,
                host_id: host.host_id,
            },
        )
        .await
        {
            Ok(_) => {
                debug!(
                    "Sent link request to tunnel for client ID {}, host ID: {} -> tunnel ID: {}",
                    client_id, host.host_id, host.tunnel_id
                );
            }
            Err(e) => match e {
                MessageError::IoError(err)
                    if err.kind() == io::ErrorKind::BrokenPipe
                        || err.kind() == io::ErrorKind::ConnectionReset =>
                {
                    debug!("Tunnel disconnected while sending link request.");
                    end_tunnel(&host_service, &mut tunnel_service, host.tunnel_id).await;
                    end_client(
                        &client_service,
                        client_id,
                        "Tunnel disconnected while sending link request.",
                    )
                    .await;
                }
                _ => {
                    debug!("Error while sending link request: {:?}", e);
                    end_tunnel(&host_service, &mut tunnel_service, host.tunnel_id).await;
                    end_client(&client_service, client_id, "Could not connect to tunnel.").await;
                }
            },
        }
    }
}

async fn end_tunnel(
    host_service: &TaskService<HostList>,
    tunnel_service: &mut TunnelList,
    tunnel_id: Uuid,
) {
    host_service.lock().await.unregister_by_tunnel(tunnel_id);
    tunnel_service.remove_tunnel(tunnel_id);
}

async fn end_client(client_service: &TaskService<ClientList>, client_id: Uuid, reason: &str) {
    let mut client = client_service.lock().await.release(client_id).unwrap();
    respond_and_close(&mut client.stream, reason).await;
}
