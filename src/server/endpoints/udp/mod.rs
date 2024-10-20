use std::{
    io::{Error, ErrorKind},
    sync::Arc,
};

use configuration::UdpEndpointConfig;
use log::{debug, error, info};
use tokio::io::Result;
use tokio::net::UdpSocket;

use crate::{
    common::{channel::RequestReceiver, connection::ConnectionStream},
    server::services::Services,
};

use super::messages::EndpointChannelRequest;

mod channel_handler;
pub mod configuration;
mod data_handler;

pub async fn start(
    services: Arc<Services>,
    name: String,
    config: UdpEndpointConfig,
    mut channel_rx: RequestReceiver<EndpointChannelRequest>,
) -> Result<()> {
    let mut listener = match UdpSocket::bind(config.get_bind_address()).await {
        Ok(listener) => ConnectionStream::from(listener),
        Err(e) => {
            error!("Failed to bind client listener: {}", e);
            return Err(Error::new(
                ErrorKind::Other,
                "Failed to bind client listener",
            ));
        }
    };

    loop {
        tokio::select! {
            request = channel_rx.wait_for_requests() => {
                match request {
                    Some(request) => {
                        debug!("Received endpoint message");
                        if let Err(e) = channel_handler::handle(request, &config).await {
                            error!("Failed to handle endpoint message: {}", e);
                        }
                    },
                    None => {
                        info!("Endpoint '{}' channel has been shutdown", name);
                        return Ok(());
                    }
                }
            }
            _ = listener.wait_for_data() => {
                if let Err(e) = data_handler::handle(&mut listener, &name, &config, &services).await {
                    error!("Failed to handle data: {}", e);
                }
            }
        }
    }
}
