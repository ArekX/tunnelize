use log::{debug, info};
use std::{
    collections::HashMap,
    io::{Error, ErrorKind},
    net::ToSocketAddrs,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
};
use tokio::{
    io::{self, Result},
    net::TcpStream,
    signal,
    sync::Mutex,
};

use crate::{
    configuration::TunnelConfiguration,
    messages::{
        self, write_message, MessageError, ServerMessage, TunnelClientRequest, TunnelMessage,
    },
};

fn resolve_address(address: String) -> Result<std::net::SocketAddr> {
    match address.to_socket_addrs() {
        Ok(mut iter) => match iter.next() {
            Some(addr) => Ok(addr),
            None => Err(Error::new(ErrorKind::InvalidInput, "Invalid address")),
        },
        Err(e) => Err(e),
    }
}

pub async fn start_client(config: TunnelConfiguration) -> Result<()> {
    let server_ip = resolve_address(config.server_address.clone())?;

    let tunnel_id = Arc::new(AtomicU32::new(0));

    let link_id_forward_map = Arc::new(Mutex::new(HashMap::<u32, String>::new()));

    let mut server = match TcpStream::connect(server_ip.clone()).await {
        Ok(stream) => stream,
        Err(e) if e.kind() == io::ErrorKind::ConnectionRefused => {
            info!(
                "Connection refused by server at {} ({})",
                config.server_address, server_ip
            );
            return Err(e);
        }
        Err(e) => {
            debug!("Error connecting to server: {:?}", e);
            return Err(e);
        }
    };

    info!(
        "Connected to server at {} ({})",
        config.server_address, server_ip
    );

    let client_requests = config
        .hostnames
        .iter()
        .map(|h| TunnelClientRequest {
            name: h.name.clone(),
            forward_address: h.forward_address.clone(),
        })
        .collect();

    match messages::write_message(&mut server, &TunnelMessage::Connect { client_requests }).await {
        Ok(_) => {}
        Err(e) => {
            debug!("Error while connecting {:?}", e);
            info!("Error connecting to server.");
            return Err(Error::new(ErrorKind::Other, "Error connecting to server"));
        }
    }

    println!(
        "Proxying addresses: {}",
        config
            .hostnames
            .iter()
            .map(|h| h.forward_address.clone())
            .collect::<Vec<String>>()
            .join(", ")
    );

    let tunnel_id_handler = tunnel_id.clone();

    tokio::spawn(async move {
        if let Err(e) = signal::ctrl_c().await {
            debug!("Error while waiting for ctrl+c signal: {:?}", e);
            return;
        }

        let mut server = TcpStream::connect(server_ip).await.unwrap();
        let tunnel_id = tunnel_id_handler.load(Ordering::SeqCst);

        if let Err(e) = write_message(&mut server, &TunnelMessage::Disconnect { tunnel_id }).await {
            debug!("Error while disconnecting: {:?}", e);
        }
    });

    loop {
        info!("Waiting for messages.");
        server.readable().await?;
        info!("Message received, processing.");

        let message: ServerMessage = match messages::read_message(&mut server).await {
            Ok(message) => message,
            Err(e) => match e {
                MessageError::ConnectionClosed => {
                    info!("Server Connection closed.");
                    return Ok(());
                }
                _ => {
                    debug!("Error while parsing {:?}", e);
                    info!("Failed to parse a message.");
                    continue;
                }
            },
        };

        let server_address = config.server_address.clone();

        let tunnel_id = tunnel_id.clone();
        let link_id_forward_map = link_id_forward_map.clone();

        tokio::spawn(async move {
            match message {
                ServerMessage::ConnectAccept {
                    tunnel_id: id,
                    resolved_links,
                } => {
                    info!("Server connect accepted. Received Tunnel ID: {}", id);

                    {
                        let mut link_id_forward_map = link_id_forward_map.lock().await;
                        for link in resolved_links {
                            info!(
                                "Proxying {} -> {} (Link ID: {})",
                                link.forward_address, link.client_address, link.link_id
                            );
                            link_id_forward_map.insert(link.link_id, link.forward_address);
                        }
                    }

                    tunnel_id.store(id, Ordering::SeqCst);
                }
                ServerMessage::LinkRequest { id, link_id } => {
                    let forward_from_address = {
                        let link_id_forward_map = link_id_forward_map.lock().await;
                        match link_id_forward_map.get(&link_id) {
                            Some(address) => address.clone(),
                            None => {
                                debug!("Link ID not found: {}", link_id);
                                return;
                            }
                        }
                    };

                    info!(
                        "Link request received. Forwarding {} -> {}",
                        forward_from_address, link_id
                    );

                    let mut tunnel = match TcpStream::connect(server_address).await {
                        Ok(stream) => stream,
                        Err(e) => {
                            debug!("Error connecting to server for proxying: {:?}", e);
                            return;
                        }
                    };
                    let mut proxy = match TcpStream::connect(forward_from_address).await {
                        Ok(stream) => stream,
                        Err(e) => {
                            debug!("Error connecting to forward address: {:?}", e);
                            return;
                        }
                    };

                    if let Err(e) = write_message(
                        &mut tunnel,
                        &TunnelMessage::LinkAccept {
                            id,
                            tunnel_id: tunnel_id.load(Ordering::SeqCst),
                        },
                    )
                    .await
                    {
                        debug!("Error while sending link accept: {:?}", e);
                        return;
                    }

                    info!("Proxying data...");
                    match io::copy_bidirectional(&mut tunnel, &mut proxy).await {
                        Ok(_) => {
                            info!("Proxying data completed...");
                        }
                        Err(e) => {
                            debug!("Error while proxying: {:?}", e);
                        }
                    }
                }
            }
        });
    }
}
