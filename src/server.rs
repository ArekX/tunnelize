use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

use log::{debug, info};
use tokio::{
    io::{self, AsyncWriteExt, Result},
    net::TcpListener,
    task::JoinHandle,
};

use crate::{
    configuration::{ServerConfiguration, ServerType},
    data::{
        client::{create_client_list, MainClientList},
        tunnel::{create_tunnel_list, MainTunnelList},
    },
    messages::{read_message, write_message, ResolvedLink, ServerMessage, TunnelMessage},
    servers::http::start_http_server,
};

async fn listen_to_tunnel(
    tunnel_port: u16,
    client_list: MainClientList,
    tunnel_list: MainTunnelList,
) -> Result<()> {
    let link = TcpListener::bind(format!("0.0.0.0:{}", tunnel_port))
        .await
        .unwrap();
    let tunel_id_counter = Arc::new(AtomicU32::new(0));

    info!("Listening to tunnel connections on 0.0.0.0:{}", tunnel_port);

    loop {
        let (mut stream, address) = link.accept().await.unwrap();

        info!("Link established with {}", address);

        stream.readable().await.unwrap();

        let tunnel_list = tunnel_list.clone();
        let client_list = client_list.clone();
        let tunel_id_counter = tunel_id_counter.clone();

        tokio::spawn(async move {
            let message: TunnelMessage = if let Ok(m) = read_message(&mut stream).await {
                m
            } else {
                return;
            };

            match message {
                TunnelMessage::Connect { client_requests } => {
                    let id = tunel_id_counter.fetch_add(1, Ordering::SeqCst);
                    let mut link_id: u32 = 0;
                    let mut resolved_links: Vec<ResolvedLink> = vec![];

                    for client_request in client_requests {
                        info!(
                            "Tunnel connected for hostname '{}' (ID: {}), waiting for client link requests.",
                            client_request.forward_address,
                            id
                        );
                        resolved_links.push(ResolvedLink {
                            link_id,
                            forward_address: client_request.forward_address.clone(),
                            client_address: format!("client-{}.localhost:3457", link_id), // fix, should be resolved to unique name
                        });

                        link_id = link_id.wrapping_add(1);
                    }

                    match write_message(
                        &mut stream,
                        &ServerMessage::ConnectAccept {
                            tunnel_id: id,
                            resolved_links: resolved_links.clone(),
                        },
                    )
                    .await
                    {
                        Ok(_) => {
                            tunnel_list
                                .lock()
                                .await
                                .register(id, stream, &resolved_links);
                        }
                        Err(e) => {
                            debug!("Error while sending connect accept: {:?}", e);
                            return;
                        }
                    }
                }
                TunnelMessage::Disconnect { tunnel_id } => {
                    info!("Tunnel disconnected for ID: {}", tunnel_id);
                    tunnel_list.lock().await.remove_tunnel(tunnel_id);
                }
                TunnelMessage::LinkAccept { id, tunnel_id } => {
                    let is_registered = { tunnel_list.lock().await.is_registered(tunnel_id) };
                    if !is_registered {
                        info!("Link request for non-existing tunnel ID: {}", tunnel_id);
                        return;
                    }

                    info!("Link accepted for client ID: {}", id);
                    let mut client = {
                        let mut client_list = client_list.lock().await;

                        if !client_list.contains_key(&id) {
                            return;
                        }

                        client_list.remove_entry(&id).unwrap().1
                    };

                    if let Err(e) = stream.write_all(client.initial_request.as_bytes()).await {
                        debug!("Error while sending initial request to client: {:?}", e);
                        return;
                    }

                    match io::copy_bidirectional(&mut client.stream, &mut stream).await {
                        _ => {}
                    }
                }
            }
        });
    }
}

pub async fn start_server(config: ServerConfiguration) -> Result<()> {
    let main_client_list: MainClientList = create_client_list();
    let main_tunnel_list: MainTunnelList = create_tunnel_list();

    let mut server_futures: Vec<JoinHandle<()>> = vec![];

    let tunnel_client_list = main_client_list.clone();
    let tunnel_list = main_tunnel_list.clone();
    server_futures.push(tokio::spawn(async move {
        listen_to_tunnel(config.tunnel_port, tunnel_client_list, tunnel_list)
            .await
            .unwrap();
    }));

    for server in config.servers {
        match server {
            ServerType::Http(config) => {
                info!("Starting HTTP server on port {}", config.port);
                let client_list = main_client_list.clone();
                let tunnel_list = main_tunnel_list.clone();
                server_futures.push(tokio::spawn(async move {
                    start_http_server(config, client_list, tunnel_list)
                        .await
                        .unwrap();
                }));
            }
            _ => {
                info!("Unsupported server type, skipping.");
                continue;
            }
        }
    }

    info!("Tunnelize initialized and running.");

    for server_future in server_futures {
        server_future.await.unwrap();
    }

    Ok(())
}
