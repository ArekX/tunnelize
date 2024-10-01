use log::{error, info};
use messages::HubMessage;
use tokio::io::Result;
use tokio::net::TcpListener;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

pub mod messages;

pub enum ServiceRequest {
    GetName,
}

pub struct CentralServer {
    services: Vec<Service>,
}

pub struct Service {
    service_tx: Sender<ServiceRequest>,
}

pub async fn start_hub_server(
    mut hub_receiver: Receiver<HubMessage>,
    services: Vec<Service>,
) -> Result<()> {
    let tunnel_port = 3456;

    let hub_server = tokio::spawn(async move {
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
        }
    });

    let channel_listener = tokio::spawn(async move {
        loop {
            let response = match hub_receiver.recv().await {
                Some(response) => response,
                None => {
                    break;
                }
            };

            match response {
                HubMessage::Name(name) => {
                    println!("Received name: {}", name);
                }
            }
        }
    });

    tokio::join!(hub_server, channel_listener).0?;

    Ok(())
}
