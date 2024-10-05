use std::{net::ToSocketAddrs, sync::Arc};

use log::{debug, info};
use tokio::{io, io::Result, net::TcpStream};

use super::services::Services;

fn resolve_address(address: String) -> Result<std::net::SocketAddr> {
    let addreses = address.to_socket_addrs()?;

    for addr in addreses {
        if addr.is_ipv4() {
            return Ok(addr);
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        "Address is not IPv4",
    ))
}

pub async fn start(services: Arc<Services>) -> Result<()> {
    let config = services.get_config();

    let server_ip = match resolve_address(config.hub_server_address.clone()) {
        Ok(addr) => addr.ip().to_string(),
        Err(e) => {
            debug!("Error resolving server address: {:?}", e);
            return Err(e);
        }
    };

    let mut server = match TcpStream::connect(server_ip.clone()).await {
        Ok(stream) => stream,
        Err(e) if e.kind() == io::ErrorKind::ConnectionRefused => {
            info!(
                "Connection refused by server at {} ({})",
                config.hub_server_address, server_ip
            );
            return Err(e);
        }
        Err(e) => {
            debug!("Error connecting to server: {:?}", e);
            return Err(e);
        }
    };

    println!(
        "Connected to tunnel server at {}",
        config.hub_server_address
    );

    Ok(())
}
