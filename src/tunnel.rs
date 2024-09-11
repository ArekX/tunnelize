use log::info;
use tokio::{
    io::{self, Result},
    net::TcpStream,
};

use crate::{
    configuration::TunnelConfiguration,
    messages::{self, write_message, ServerMessage, TunnelMessage},
};

const TUNNEL_FROM_ADDRESS: &str = "0.0.0.0:8000";

pub async fn start_client(config: TunnelConfiguration) -> Result<()> {
    let mut server = TcpStream::connect(config.server_address.clone()).await?;

    messages::write_message(&mut server, &TunnelMessage::Connect)
        .await
        .unwrap();

    loop {
        info!("Connected to server at {}", config.server_address);
        info!("Proxying from {}", TUNNEL_FROM_ADDRESS);
        info!("Waiting for request.");
        server.readable().await?;

        info!("Request received.");

        let message: ServerMessage = match messages::read_message(&mut server).await {
            Ok(message) => message,
            Err(e) => match e {
                messages::MessageError::ConnectionClosed => {
                    info!("Connection closed.");
                    return Ok(());
                }
                _ => {
                    info!("Error reading message: {:?}", e);
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

                    io::copy_bidirectional(&mut tunnel, &mut proxy)
                        .await
                        .unwrap();
                }
            }
        });
    }
}
