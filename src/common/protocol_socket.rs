use std::{
    io::{Error, ErrorKind},
    net::{SocketAddr, ToSocketAddrs},
};

use tokio::{
    io::Result,
    net::{TcpStream, UdpSocket},
};

pub trait ProtocolSocket
where
    Self: Sized,
{
    type Socket;
    async fn connect(address: &SocketAddr) -> Result<Self::Socket>;
}

impl ProtocolSocket for TcpStream {
    type Socket = TcpStream;
    async fn connect(address: &SocketAddr) -> Result<Self::Socket> {
        TcpStream::connect(address).await
    }
}

impl ProtocolSocket for UdpSocket {
    type Socket = UdpSocket;
    async fn connect(address: &SocketAddr) -> Result<Self::Socket> {
        UdpSocket::bind(address).await
    }
}

pub async fn connect_to_address<Protocol: ProtocolSocket>(
    address: &str,
    port: u16,
) -> Result<(Protocol::Socket, SocketAddr)> {
    let addr_port = format!("{}:{}", address, port);
    let socket_addrs: Vec<SocketAddr> = addr_port.to_socket_addrs()?.collect();

    if socket_addrs.is_empty() {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("Could not resolve address: {}", addr_port),
        ));
    }

    let mut error = None;

    for addr in socket_addrs.iter().filter(|addr| addr.is_ipv6()) {
        match Protocol::connect(addr).await {
            Ok(stream) => return Ok((stream, addr.clone())),
            Err(e) => error = Some(e),
        }
    }

    for addr in socket_addrs.iter().filter(|addr| addr.is_ipv4()) {
        match Protocol::connect(addr).await {
            Ok(stream) => return Ok((stream, addr.clone())),
            Err(e) => error = Some(e),
        }
    }

    if let Some(e) = error {
        return Err(e);
    }

    Err(Error::new(
        ErrorKind::Other,
        format!("Failed to connect to address: {}", addr_port),
    ))
}
