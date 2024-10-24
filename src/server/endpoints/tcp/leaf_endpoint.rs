use log::{debug, error};
use tokio::io::{Error, ErrorKind, Result};
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;

use crate::common::channel::RequestSender;
use crate::common::connection::ConnectionStream;

use super::messages::{ClientConnect, TcpChannelRequest};

pub async fn create_leaf_endpoint(
    port: u16,
    hub_tx: RequestSender<TcpChannelRequest>,
    cancel_token: CancellationToken,
) -> Result<()> {
    // TODO: Add proper address via config.
    let listener = match TcpListener::bind(format!("0.0.0.0:{}", port)).await {
        Ok(listener) => listener,
        Err(e) => {
            error!("Failed to bind client listener: {}", e);
            return Err(Error::new(
                ErrorKind::Other,
                "Failed to bind client listener",
            ));
        }
    };

    // TODO: Handle cancel token

    loop {
        let Ok((stream, address)) = listener.accept().await else {
            error!("Failed to accept connection.");
            continue;
        };

        debug!(
            "Accepted TCP connection from client '{}' at port {}",
            address, port
        );

        if let Err(e) = hub_tx
            .request(ClientConnect {
                stream: ConnectionStream::from(stream),
                port,
            })
            .await
        {
            error!("Failed to send leaf connection request: {:?}", e);
        }
    }
}
