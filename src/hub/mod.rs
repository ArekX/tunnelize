use log::{error, info};
use messages::HubMessage;
use requests::ServiceRequest;
use tokio::io::Result;
use tokio::net::TcpListener;
use tokio::sync::mpsc::{self, Receiver};

pub mod messages;
pub mod requests;

pub struct Service {
    pub name: String,
    pub service_tx: mpsc::Sender<ServiceRequest>,
}

pub async fn start_hub_server(
    mut hub_receiver: Receiver<HubMessage>,
    services: Vec<Service>,
) -> Result<()> {
    let tunnel_port = 3456;

    let hub_server = tokio::spawn(async move {
        start_tcp_server(tunnel_port)
            .await
            .expect("Tcp Server Failed");
    });

    let channel_listener = tokio::spawn(async move {
        start_channel_listener(hub_receiver)
            .await
            .expect("Channel listener failed");
    });

    tokio::try_join!(hub_server, channel_listener)?;

    Ok(())
}

pub async fn start_channel_listener(mut hub_receiver: Receiver<HubMessage>) -> Result<()> {
    loop {
        let response = match hub_receiver.recv().await {
            Some(response) => response,
            None => {
                break;
            }
        };

        match response {
            HubMessage::Test(name) => {
                println!("Received name: {}", name);
            }
        }
    }

    Ok(())
}

pub async fn start_tcp_server(tunnel_port: u16) -> Result<()> {
    let tunnel_listener = match TcpListener::bind(format!("0.0.0.0:{}", tunnel_port)).await {
        Ok(listener) => listener,
        Err(e) => {
            error!("Failed to bind tunnel listener: {}", e);
            return Ok(());
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
}
