use std::io::Error;
use std::net::{SocketAddr, ToSocketAddrs};

use log::{debug, error, info};
use tokio::io::{self};
use tokio::{io::Result, net::TcpStream};

use super::connection::Connection;
use super::encryption::{ClientEncryptionType, ClientTlsEncryption};

async fn connect_to_address(address: &str, port: u16) -> Result<TcpStream> {
    let addr_port = format!("{}:{}", address, port);
    let socket_addrs: Vec<SocketAddr> = addr_port.to_socket_addrs()?.collect();

    if socket_addrs.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Could not resolve address: {}", addr_port),
        ));
    }

    let mut error = None;

    for addr in socket_addrs.iter().filter(|addr| addr.is_ipv6()) {
        match TcpStream::connect(addr).await {
            Ok(stream) => return Ok(stream),
            Err(e) => error = Some(e),
        }
    }

    for addr in socket_addrs.iter().filter(|addr| addr.is_ipv4()) {
        match TcpStream::connect(addr).await {
            Ok(stream) => return Ok(stream),
            Err(e) => error = Some(e),
        }
    }

    if let Some(e) = error {
        return Err(e);
    }

    Err(Error::new(
        io::ErrorKind::Other,
        format!("Failed to connect to address: {}", addr_port),
    ))
}

pub async fn create_tcp_client(
    address: &str,
    port: u16,
    encryption: Option<ClientEncryptionType>,
) -> Result<Connection> {
    match connect_to_address(address, port).await {
        Ok(stream) => {
            match encryption {
                Some(encryption_type) => {
                    let tls = ClientTlsEncryption::new(encryption_type).await;

                    info!("Connected to (TLS) server at {}", address);

                    // TODO: needs testing and fixing
                    return Ok(tls.connect(stream, address).await?);
                }
                None => {
                    info!("Connected to server at {}", address);
                    return Ok(Connection::from(stream));
                }
            }
        }
        Err(e) if e.kind() == io::ErrorKind::ConnectionRefused => {
            error!("Connection refused by server at {}", address);
            return Err(e);
        }
        Err(e) => {
            debug!("Error connecting to server: {:?}", e);
            return Err(e);
        }
    }
}
