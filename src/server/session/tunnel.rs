use std::sync::Arc;

use super::messages::TunnelSessionMessage;
use log::{debug, info};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    common::{connection::ConnectionStream, transport::MessageError},
    server::incoming_requests::ServerRequestMessage,
};

use super::super::services::Services;

#[derive(Clone, Debug)]
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
    session: TunnelSession,
    mut stream: ConnectionStream,
    mut channel_rx: mpsc::Receiver<TunnelSessionMessage>,
) {
    let id = session.get_id();

    loop {
        let message: ServerRequestMessage;

        tokio::select! {
                data = channel_rx.recv() => {
                    info!("Got data via channel {:?}", data);
                    break;
                }
                message_result = stream.read_message::<ServerRequestMessage>() => {
                    match message_result {
                        Ok(ok_message) => {
                            message = ok_message;
                        }
                        Err(e) => match e {
                            MessageError::ConnectionClosed => {
                                info!("Tunnel {} closed connection.", id);
                                break;
                            }
                            _ => {
                                debug!("Error while parsing {:?}", e);
                                info!("Failed to read message from tunnel session {}: {}", id, e);
                                continue;
                            }


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
