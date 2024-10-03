use std::{sync::Arc, time::Duration};

use log::{debug, error, info, warn};
use tokio::{
    io::{self, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    time::timeout,
};
use uuid::Uuid;

use crate::{
    server::http::{
        http_protocol::{
            find_request_host, get_error_response, get_unauthorized_response, is_authorized,
        },
        messages::ServerMessage,
    },
    transport::{write_message, MessageError},
};

use super::services::Services;

async fn read_http_request_string(stream: &mut TcpStream) -> String {
    let mut request_buffer = Vec::new();
    let duration = Duration::from_secs(5);
    loop {
        debug!("Waiting tcp stream to be readable...");
        match timeout(duration, stream.readable()).await {
            Ok(_) => {}
            Err(_) => {
                debug!("Timeout while waiting for client stream to be readable.");
                break;
            }
        }

        let mut buffer = [0; 100024];

        match stream.try_read(&mut buffer) {
            Ok(0) => {
                break;
            }
            Ok(read) => {
                request_buffer.extend_from_slice(&buffer[..read]);
                if read < buffer.len() {
                    break;
                }
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                break;
            }
            Err(e) => {
                debug!("Error while reading until block: {:?}", e);
                break;
            }
        }
    }

    match String::from_utf8(request_buffer) {
        Ok(result) => result,
        Err(e) => {
            debug!("Error while converting buffer to string: {:?}", e);
            String::new()
        }
    }
}

async fn respond_and_close(stream: &mut TcpStream, response: &String) {
    let duration = Duration::from_secs(5);

    debug!("Writing error response to client...");
    if let Err(e) = timeout(duration, stream.write_all(response.as_bytes())).await {
        error!("Failed to respond to client: {}", e);
    }

    debug!("Flushing error response to client...");
    if let Err(e) = stream.flush().await {
        error!("Failed to flush stream: {}", e);
    }

    debug!("Shutting down client stream...");
    if let Err(e) = stream.shutdown().await {
        error!("Failed to close client connection: {}", e);
    }

    debug!("Client connection closed.");
}

async fn wait_for_client_readable(stream: &mut TcpStream, wait_seconds: u16) -> bool {
    let duration = Duration::from_secs(wait_seconds.into());
    match timeout(duration, stream.readable()).await {
        Ok(_) => true,
        Err(_) => {
            debug!("Timeout while waiting for client stream to be readable.");
            false
        }
    }
}

pub async fn start(services: Arc<Services>) {
    let config = services.get_config();
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
        if !wait_for_client_readable(&mut stream, config.max_client_input_wait).await {
            warn!("Client stream not readable or pre-connection without sending data, closing connection.");
            if let Err(e) = stream.shutdown().await {
                debug!("Error while closing client stream. {:?}", e);
            }
            continue;
        }

        let http_request = read_http_request_string(&mut stream).await;

        let hostname = if let Some(hostname) = find_request_host(&http_request) {
            hostname
        } else {
            info!("No hostname found in initial request, closing connection.");
            respond_and_close(
                &mut stream,
                &get_error_response(&http_request, "No hostname found for this request. Cannot resolve to a tunnel. Closing connection.".to_owned())
            ).await;
            continue;
        };

        info!("Resolved hostname '{}' from initial request", hostname);
        let host = {
            let host_service = services.get_host_service().await;
            match host_service.find_host(&hostname) {
                Some(host) => host,
                None => {
                    error!("Failed to find host for hostname '{}'", hostname);
                    respond_and_close(
                        &mut stream,
                        &get_error_response(
                            &http_request,
                            "No tunnel connected for this hostname. Closing connection.".to_owned(),
                        ),
                    )
                    .await;
                    continue;
                }
            }
        };

        let client_id = {
            let mut client_service = services.get_client_service().await;
            client_service.issue_client_id()
        };

        {
            info!(
                "Client connected from {}, assigned ID: {}",
                address, client_id
            );
            services
                .get_client_service()
                .await
                .register(client_id, stream, http_request.clone());
        }

        let mut tunnel_service = services.get_tunnel_service().await;

        let tunnel = match tunnel_service.get_by_id(host.tunnel_id) {
            Some(tunnel) => tunnel,
            None => {
                error!("Failed to find tunnel for ID: {}", host.tunnel_id);
                end_client(
                    &services,
                    client_id,
                    &get_error_response(
                        &http_request,
                        "No tunnel connected for this hostname. Closing connection.".to_owned(),
                    ),
                )
                .await;
                continue;
            }
        };

        if let Some(user) = tunnel.client_authorization.as_ref() {
            debug!("Checking client authorization for client ID {}", client_id);
            if !is_authorized(&http_request, &user.username, &user.password) {
                info!(
                    "Unauthorized client connection from {}, closing connection.",
                    address
                );
                end_client(
                    &services,
                    client_id,
                    &get_unauthorized_response(&http_request, &user.realm),
                )
                .await;
                continue;
            }
        }

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
                    end_tunnel(&services, host.tunnel_id).await;
                    end_client(
                        &services,
                        client_id,
                        &get_error_response(
                            &http_request,
                            "Tunnel disconnected while sending link request.".to_owned(),
                        ),
                    )
                    .await;
                }
                _ => {
                    debug!("Error while sending link request: {:?}", e);
                    end_tunnel(&services, host.tunnel_id).await;
                    end_client(
                        &services,
                        client_id,
                        &get_error_response(
                            &http_request,
                            "Could not connect to tunnel.".to_owned(),
                        ),
                    )
                    .await;
                }
            },
        }
    }
}

async fn end_tunnel(services: &Arc<Services>, tunnel_id: Uuid) {
    services
        .get_host_service()
        .await
        .unregister_by_tunnel(tunnel_id);
    services.get_tunnel_service().await.remove_tunnel(tunnel_id);
}

async fn end_client(services: &Arc<Services>, client_id: Uuid, response: &String) {
    let mut client = services
        .get_client_service()
        .await
        .release(client_id)
        .unwrap();
    respond_and_close(&mut client.stream, response).await;
}
