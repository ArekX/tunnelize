use std::sync::Arc;

use super::messages::TunnelSessionMessage;
use log::info;
use tokio::{
    io::{self},
    net::TcpStream,
    sync::mpsc,
};
use uuid::Uuid;

use crate::{common::transport::read_message, server::messages::ServerRequestMessage};

use super::super::services::Services;

pub struct TunnelSession {
    id: Uuid,
    has_admin_privileges: bool,
    channel_tx: mpsc::Sender<TunnelSessionMessage>,
}

impl TunnelSession {
    pub fn new(has_admin_privileges: bool, channel_tx: mpsc::Sender<TunnelSessionMessage>) -> Self {
        let id = Uuid::new_v4();
        Self {
            id,
            has_admin_privileges,
            channel_tx,
        }
    }

    pub fn get_channel_tx(&self) -> mpsc::Sender<TunnelSessionMessage> {
        self.channel_tx.clone()
    }

    pub fn get_id(&self) -> Uuid {
        self.id
    }
}

pub fn create(has_admin_privileges: bool) -> (TunnelSession, mpsc::Receiver<TunnelSessionMessage>) {
    let (channel_tx, channel_rx) = mpsc::channel::<TunnelSessionMessage>(100);

    (
        TunnelSession::new(has_admin_privileges, channel_tx),
        channel_rx,
    )
}

pub async fn start(
    services: Arc<Services>,
    mut stream: TcpStream,
    mut channel_rx: mpsc::Receiver<TunnelSessionMessage>,
) {
    let id = Uuid::new_v4();

    loop {
        let message: ServerRequestMessage;

        tokio::select! {
            data = channel_rx.recv() => {
                info!("Got data via channel {:?}", data);
                break;
            }
            message_result = read_message::<TcpStream, ServerRequestMessage>(&mut stream) => {
                match message_result {
                    Ok(ok_message) => {
                        message = ok_message;
                    }
                    Err(e) => {
                        info!("Failed to read message from tunnel session {}: {}", id, e);
                        break;
                    }
                }
            }

        }

        println!("Received message from tunnel session {}: {:?}", id, message);

        println!("Tunnel session {} is running.", id);
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        // TODO: Implement the rest of the tunnel session logic
    }
}

async fn is_closed(server: &mut TcpStream) -> bool {
    let mut buf = [0; 1];
    match server.peek(&mut buf).await {
        Ok(0) => true,
        Ok(_) => false,
        Err(e) if e.kind() == io::ErrorKind::WouldBlock => false,
        Err(_) => true,
    }
}
