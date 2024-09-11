use log::{debug, info};
use std::{
    io::{Error, ErrorKind},
    net::ToSocketAddrs,
};
use tokio::{
    io::{self, Result},
    net::TcpStream,
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
            hostname: "localhost:3457".to_string(),
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
                    info!("Connection closed.");
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

        tokio::spawn(async move {
            match message {
                ServerMessage::LinkRequest { id } => {
                    let mut tunnel = TcpStream::connect(server_address).await.unwrap();
                    let mut proxy = TcpStream::connect(TUNNEL_FROM_ADDRESS).await.unwrap();

                    write_message(&mut tunnel, &TunnelMessage::LinkAccept { id })
                        .await
                        .unwrap();

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
