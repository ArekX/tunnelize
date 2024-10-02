use std::sync::Arc;
use std::time::Duration;

use log::{debug, error, info};
use tokio::io::Result;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::Sender;
use tokio::time::timeout;

use crate::transport::read_message;

use super::messages::{HubMessage, TunnelServerMessage};
use super::services::Services;

pub async fn start(services: Arc<Services>) -> Result<()> {
    let hub_config = services.get_config();
    let hub_tx = services.get_hub_tx();

    let tunnel_listener =
        match TcpListener::bind(format!("0.0.0.0:{}", hub_config.server_port)).await {
            Ok(listener) => listener,
            Err(e) => {
                error!("Failed to bind tunnel listener: {}", e);
                return Ok(());
            }
        };

    info!(
        "Listening to tunnel connections on 0.0.0.0:{}",
        hub_config.server_port
    );

    loop {
        let (mut stream, address) = match tunnel_listener.accept().await {
            Ok(stream_pair) => stream_pair,
            Err(e) => {
                error!("Failed to accept tunnel connection: {}", e);
                continue;
            }
        };

        info!("Tunnel connected at: {}", address);

        if !wait_for_tunnel_readable(&mut stream, hub_config.max_tunnel_input_wait).await {
            continue;
        }

        let hub_tx = hub_tx.clone();

        tokio::spawn(async move {
            process_request(stream, hub_tx).await;
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

async fn process_request(mut stream: TcpStream, hub_tx: Sender<HubMessage>) {
    let message: TunnelServerMessage = match read_message(&mut stream).await {
        Ok(message) => message,
        Err(e) => {
            debug!("Error while reading tunnel message: {:?}", e);
            return;
        }
    };

    match message {
        TunnelServerMessage::Tunnel(tunnel_message) => {
            if let Err(e) = hub_tx.send(HubMessage::Tunnel(tunnel_message)).await {
                debug!("Error sending tunnel message to hub: {:?}", e);
            }
        }
        _ => {
            debug!("Received unknown message");
        }
    }
}
