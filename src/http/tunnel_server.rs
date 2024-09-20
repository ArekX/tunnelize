use log::{debug, error, info};
use tokio::{
    io::{self, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use crate::{
    http::tunnel_helper::disconnect_tunnel,
    transport::{read_message, write_message},
};

use super::{
    client_list::ClientList,
    host_list::HostList,
    messages::{ResolvedLink, ServerMessage, TunnelMessage},
    tunnel_list::{RequestedProxy, TunnelList},
    HttpServerConfig, TaskService,
};

pub async fn start_tunnel_server(
    config: HttpServerConfig,
    host_service: TaskService<HostList>,
    tunnel_service: TaskService<TunnelList>,
    client_service: TaskService<ClientList>,
) {
    let tunnel_listener = match TcpListener::bind(format!("0.0.0.0:{}", config.tunnel_port)).await {
        Ok(listener) => listener,
        Err(e) => {
            error!("Failed to bind tunnel listener: {}", e);
            return;
        }
    };

    info!(
        "Listening to tunnel connections on 0.0.0.0:{}",
        config.tunnel_port
    );

    loop {
        let (stream, address) = match tunnel_listener.accept().await {
            Ok(stream_pair) => stream_pair,
            Err(e) => {
                error!("Failed to accept tunnel connection: {}", e);
                continue;
            }
        };

        info!("Link established with {}", address);

        if let Err(e) = stream.readable().await {
            error!("Failed to read from tunnel connection: {}", e);
            continue;
        }

        let tunnel_service = tunnel_service.clone();
        let client_service = client_service.clone();
        let host_service = host_service.clone();

        tokio::spawn(async move {
            process_tunnel_request(stream, host_service, tunnel_service, client_service).await;
        });
    }
}

async fn process_tunnel_request(
    mut stream: TcpStream,
    host_service: TaskService<HostList>,
    tunnel_service: TaskService<TunnelList>,
    client_service: TaskService<ClientList>,
) {
    let message: TunnelMessage = if let Ok(m) = read_message(&mut stream).await {
        m
    } else {
        return;
    };

    match message {
        TunnelMessage::Connect { proxies } => {
            let mut host_service = host_service.lock().await;

            let id = {
                let mut tunnel_service = tunnel_service.lock().await;
                tunnel_service.issue_tunnel_id()
            };

            let mut requested_proxies: Vec<RequestedProxy> = vec![];
            let mut resolved_links: Vec<ResolvedLink> = vec![];

            for proxy in proxies {
                let resolved_host = host_service.register(id, proxy.desired_name);

                resolved_links.push(ResolvedLink {
                    host_id: resolved_host.host_id,
                    forward_address: proxy.forward_address.clone(),
                    hostname: resolved_host.hostname.clone(),
                });

                requested_proxies.push(RequestedProxy {
                    resolved_host,
                    forward_address: proxy.forward_address,
                });
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
                    let mut tunnel_service = tunnel_service.lock().await;
                    tunnel_service.register(id, stream, requested_proxies);
                }
                Err(e) => {
                    debug!("Error while sending connect accept: {:?}", e);
                    return;
                }
            }
        }
        TunnelMessage::Disconnect { tunnel_id } => {
            info!("Tunnel disconnected for ID: {}", tunnel_id);
            disconnect_tunnel(&host_service, &tunnel_service, tunnel_id).await;
        }
        TunnelMessage::ClientLinkAccept {
            client_id,
            tunnel_id,
        } => {
            let is_registered = { tunnel_service.lock().await.is_registered(tunnel_id) };
            if !is_registered {
                info!("Link request for non-existing tunnel ID: {}", tunnel_id);
                return;
            }

            info!("Link accepted for client ID: {}", client_id);
            let mut client = {
                let mut client_service = client_service.lock().await;

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
        TunnelMessage::ClientLinkDeny {
            tunnel_id,
            client_id,
            reason,
        } => {
            let is_registered = { tunnel_service.lock().await.is_registered(tunnel_id) };
            if !is_registered {
                info!("Link deny for non-existing tunnel ID: {}", tunnel_id);
                return;
            }
            info!(
                "Link denied for client ID: {}. Reason: {}",
                client_id, reason
            );

            let mut client = {
                let mut client_service = client_service.lock().await;

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
    }
}
