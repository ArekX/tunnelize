use std::{net::SocketAddr, sync::Arc};

use log::{debug, error};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::mpsc::Sender,
};
use tokio_util::sync::CancellationToken;

use crate::{
    common::transport::{read_message, write_message},
    server::messages::{ServerRequestMessage, ServerResponseMessage},
};

use super::{messages::ChannelMessage, services::Services};

pub async fn start(services: Arc<Services>, cancel_token: CancellationToken) {
    let config = services.get_config();

    let listener = match TcpListener::bind(format!("0.0.0.0:{}", config.server_port)).await {
        Ok(listener) => listener,
        Err(e) => {
            error!("Failed to bind client listener: {}", e);
            return;
        }
    };

    loop {
        let mut stream: TcpStream;
        let address: SocketAddr;

        tokio::select! {
            _ = cancel_token.cancelled() => {
                debug!("Hub server stopped.");
                return;
            }
            client = listener.accept() => {
                (stream, address) = match client {
                    Ok(stream_pair) => stream_pair,
                    Err(e) => {
                        error!("Failed to accept client connection: {}", e);
                        continue;
                    }
                };
            }
        }

        let message: ServerRequestMessage = match read_message(&mut stream).await {
            Ok(message) => message,
            Err(e) => {
                error!("Failed to read message from client: {}", e);
                continue;
            }
        };

        println!("message: {:?}", message);

        match write_message(
            &mut stream,
            &ServerResponseMessage::AuthTunnelAccepted {
                tunnel_id: "opopop".to_owned(),
            },
        )
        .await
        {
            Ok(_) => (),
            Err(e) => {
                error!("Failed to write message to client: {}", e);
                continue;
            }
        }
    }
}
