use log::{debug, info};
use std::{
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
};

use crate::{
    configuration::TunnelConfiguration,
    messages::{self, write_message, MessageError, ServerMessage, TunnelMessage},
};

const TUNNEL_FROM_ADDRESS: &str = "0.0.0.0:8000";

pub async fn start_client(config: TunnelConfiguration) -> Result<()> {
    let server_ip = config
        .server_address
        .clone()
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();

    let tunnel_id = Arc::new(AtomicU32::new(0));

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

    match messages::write_message(
        &mut server,
        &TunnelMessage::Connect {
            hostname: config.hostname.clone(),
        },
    )
    .await
    {
        Ok(_) => {}
        Err(e) => {
            debug!("Error while connecting {:?}", e);
            info!("Error connecting to server.");
            return Err(Error::new(ErrorKind::Other, "Error connecting to server"));
        }
    }

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
        info!(
            "Connected to server at {} ({})",
            config.server_address, server_ip
        );
        info!("Proxying from {}", TUNNEL_FROM_ADDRESS);
        info!("Waiting for request.");
        server.readable().await?;

        info!("Request received.");

        let message: ServerMessage = match messages::read_message(&mut server).await {
            Ok(message) => message,
            Err(e) => match e {
                MessageError::ConnectionClosed => {
                    info!("Server connection closed.");
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

        tokio::spawn(async move {
            match message {
                ServerMessage::ConnectAccept { tunnel_id: id } => {
                    info!("Server connect accepted. Received Tunnel ID: {}", id);
                    tunnel_id.store(id, Ordering::SeqCst);
                }
                ServerMessage::LinkRequest { id } => {
                    let mut tunnel = TcpStream::connect(server_address).await.unwrap();
                    let mut proxy = TcpStream::connect(TUNNEL_FROM_ADDRESS).await.unwrap();

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

                    match io::copy_bidirectional(&mut tunnel, &mut proxy).await {
                        Ok(_) => {}
                        Err(e) => {
                            debug!("Error while proxying: {:?}", e);
                        }
                    }
                }
            }
        });
    }
}
