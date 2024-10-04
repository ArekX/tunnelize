use std::{sync::Arc, time::Duration};

use log::{debug, error, info};
use tokio::{
    io::{self, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    time::timeout,
};
use uuid::Uuid;

use crate::{
    transport::{read_message, write_message},
    tunnel::http::messages::{HttpTunnelMessage, Proxy},
};

use super::{
    messages::{ResolvedLink, ServerMessage},
    services::Services,
    tunnel_list::RequestedProxy,
    ClientAuthorizeUser,
};

pub async fn start_tunnel_server(services: Arc<Services>) {
    let tunnel_port: u16 = 3456; // TODO: This will be in hub anyway.
    let tunnel_listener = match TcpListener::bind(format!("0.0.0.0:{}", tunnel_port)).await {
        Ok(listener) => listener,
        Err(e) => {
            error!("Failed to bind tunnel listener: {}", e);
            return;
        }
    };

    info!("Listening to tunnel connections on 0.0.0.0:{}", tunnel_port);

    loop {
        let (mut stream, address) = match tunnel_listener.accept().await {
            Ok(stream_pair) => stream_pair,
            Err(e) => {
                error!("Failed to accept tunnel connection: {}", e);
                continue;
            }
        };

        info!("Tunnel connected at: {}", address);

        if !wait_for_tunnel_readable(&mut stream, 10).await {
            continue;
        }

        let services = services.clone();

        tokio::spawn(async move {
            process_tunnel_request(stream, services).await;
        });
    }
}

async fn wait_for_tunnel_readable(stream: &mut TcpStream, wait_seconds: u16) -> bool {
    let duration = Duration::from_secs(wait_seconds.into());
    match timeout(duration, stream.readable()).await {
        Ok(_) => true,
        Err(_) => {
            debug!("Timeout while waiting for tunnel stream to be readable.");
            false
        }
    }
}

async fn process_tunnel_request(mut stream: TcpStream, services: Arc<Services>) {
    let message: HttpTunnelMessage = match read_message(&mut stream).await {
        Ok(message) => message,
        Err(e) => {
            debug!("Error while reading tunnel message: {:?}", e);
            return;
        }
    };

    match message {
        HttpTunnelMessage::Connect {
            proxies,
            tunnel_auth_key,
            client_authorization,
        } => {
            process_tunnel_connect(
                &services,
                proxies,
                tunnel_auth_key,
                client_authorization,
                stream,
            )
            .await
        }
        HttpTunnelMessage::Disconnect { tunnel_id } => {
            process_disconnect_tunnel(&services, tunnel_id).await
        }
        HttpTunnelMessage::ClientLinkAccept {
            client_id,
            tunnel_id,
        } => process_client_accept(&services, tunnel_id, client_id, stream).await,
        HttpTunnelMessage::ClientLinkDeny {
            tunnel_id,
            client_id,
            reason,
        } => process_client_deny(&services, tunnel_id, client_id, reason).await,
    };
}

async fn validate_tunnel_id(services: &Arc<Services>, tunnel_id: Uuid) -> bool {
    let is_valid = services.get_tunnel_service().await.is_registered(tunnel_id);

    if !is_valid {
        info!("Invalid tunnel ID: {}", tunnel_id);
    }

    is_valid
}

async fn process_disconnect_tunnel(services: &Arc<Services>, tunnel_id: Uuid) {
    if !validate_tunnel_id(services, tunnel_id).await {
        return;
    }

    info!("Tunnel disconnected for ID: {}", tunnel_id);
    services
        .get_host_service()
        .await
        .unregister_by_tunnel(tunnel_id);
    services.get_tunnel_service().await.remove_tunnel(tunnel_id);
}

async fn process_client_deny(
    services: &Arc<Services>,
    tunnel_id: Uuid,
    client_id: Uuid,
    reason: String,
) {
    if !validate_tunnel_id(services, tunnel_id).await {
        return;
    }

    info!(
        "Link denied for client ID: {}. Reason: {}",
        client_id, reason
    );
    let mut client = {
        let mut client_service = services.get_client_service().await;

        if !client_service.is_registered(client_id) {
            debug!("Client ID is not registered: {}", client_id);
            return;
        }

        match client_service.release(client_id) {
            Some(client) => client,
            None => {
                debug!("Client ID could not be acquired: {}", client_id);
                return;
            }
        }
    };
    if let Err(e) = client.stream.write_all(reason.as_bytes()).await {
        debug!("Error while sending link deny reason: {:?}", e);
    }

    if let Err(e) = client.stream.shutdown().await {
        debug!("Error while shutting down client stream: {:?}", e);
    }
}

async fn process_client_accept(
    services: &Arc<Services>,
    tunnel_id: Uuid,
    client_id: Uuid,
    mut stream: TcpStream,
) {
    if !validate_tunnel_id(services, tunnel_id).await {
        return;
    }

    info!("Link accepted for client ID: {}", client_id);
    let mut client = {
        let mut client_service = services.get_client_service().await;

        if !client_service.is_registered(client_id) {
            debug!("Client ID is not registered: {}", client_id);
            return;
        }

        match client_service.release(client_id) {
            Some(client) => client,
            None => {
                debug!("Client ID could not be acquired: {}", client_id);
                return;
            }
        }
    };
    if let Err(e) = stream.write_all(client.initial_request.as_bytes()).await {
        debug!("Error while sending initial request to client: {:?}", e);
        return;
    }
    debug!(
        "Tunnel link established for client ID: {}, sending data...",
        client_id
    );

    match io::copy_bidirectional(&mut client.stream, &mut stream).await {
        _ => {
            println!("Client {} tunnel link closed.", client_id);
        }
    }
}

async fn process_tunnel_connect(
    services: &Arc<Services>,
    proxies: Vec<Proxy>,
    auth_key: Option<String>,
    client_authorization: Option<ClientAuthorizeUser>,
    mut stream: TcpStream,
) {
    let config = services.get_config();

    if let Some(key) = &config.tunnel_auth_key {
        let tunnel_key = match auth_key {
            Some(key) => key,
            None => String::new(),
        };
        if tunnel_key != *key {
            debug!("Invalid auth key: {:?}", tunnel_key);

            write_message(
                &mut stream,
                &ServerMessage::TunnelDeny {
                    reason: "Invalid auth key provided".to_string(),
                },
            )
            .await
            .unwrap();
            stream.shutdown().await.unwrap();
            return;
        }
    }

    let mut host_service = services.get_host_service().await;
    let id = {
        let tunnel_service = services.get_tunnel_service().await;
        tunnel_service.issue_tunnel_id()
    };
    let mut requested_proxies: Vec<RequestedProxy> = vec![];
    let mut resolved_links: Vec<ResolvedLink> = vec![];
    for proxy in proxies {
        let resolved_host = host_service.register(id, proxy.desired_name);

        info!(
            "Registered host {} for tunnel ID: {}",
            resolved_host.hostname, id
        );

        let url = config
            .tunnel_url_template
            .replace("{hostname}", &resolved_host.hostname);

        resolved_links.push(ResolvedLink {
            host_id: resolved_host.host_id,
            forward_address: proxy.forward_address.clone(),
            hostname: resolved_host.hostname.clone(),
            url,
        });

        requested_proxies.push(RequestedProxy { resolved_host });
    }

    match write_message(
        &mut stream,
        &ServerMessage::TunnelAccept {
            tunnel_id: id,
            resolved_links,
        },
    )
    .await
    {
        Ok(_) => {
            let mut tunnel_service = services.get_tunnel_service().await;
            tunnel_service.register(id, stream, requested_proxies, client_authorization);
        }
        Err(e) => {
            debug!("Error while sending connect accept: {:?}", e);
            return;
        }
    }
}
