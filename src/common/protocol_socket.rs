use std::{
    io::{Error, ErrorKind},
    net::{SocketAddr, ToSocketAddrs},
};

use tokio::{io::Result, net::TcpStream};

pub trait ProtocolSocket
where
    Self: Sized,
{
    type Socket;
    type Context;
    async fn connect(address: &SocketAddr, context: &Self::Context) -> Result<Self::Socket>;
}

impl ProtocolSocket for TcpStream {
    type Socket = TcpStream;
    type Context = ();
    async fn connect(address: &SocketAddr, _context: &Self::Context) -> Result<Self::Socket> {
        TcpStream::connect(address).await
    }
}

pub async fn connect_to_address<Protocol: ProtocolSocket>(
    address: &str,
    port: u16,
    context: Protocol::Context,
) -> Result<(Protocol::Socket, SocketAddr)> {
    let addr_port = format!("{address}:{port}");
    let socket_addrs: Vec<SocketAddr> = addr_port.to_socket_addrs()?.collect();

    if socket_addrs.is_empty() {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("Could not resolve address: {addr_port}"),
        ));
    }

    let mut error = None;

    for addr in socket_addrs.iter().filter(|addr| addr.is_ipv6()) {
        match Protocol::connect(addr, &context).await {
            Ok(stream) => return Ok((stream, *addr)),
            Err(e) => error = Some(e),
        }
    }

    for addr in socket_addrs.iter().filter(|addr| addr.is_ipv4()) {
        match Protocol::connect(addr, &context).await {
            Ok(stream) => return Ok((stream, *addr)),
            Err(e) => error = Some(e),
        }
    }

    if let Some(e) = error {
        return Err(e);
    }

    Err(Error::other(
        format!("Failed to connect to address: {addr_port}"),
    ))
}
