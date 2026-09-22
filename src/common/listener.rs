use std::{
    io::{Error, ErrorKind},
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
};

use log::debug;
use socket2::{Domain, Protocol, Socket, Type};
use tokio::{
    io::Result,
    net::{TcpListener, UdpSocket, lookup_host},
};

/// Resolves a configured bind address ("0.0.0.0", "::", "127.0.0.1", "localhost",
/// ...) into the socket address to bind. IP literals never touch the resolver.
async fn resolve_bind_address(address: &str, port: u16) -> Result<SocketAddr> {
    let host = address.trim_start_matches('[').trim_end_matches(']');

    if let Ok(ip) = host.parse::<IpAddr>() {
        return Ok(SocketAddr::new(ip, port));
    }

    lookup_host((host, port)).await?.next().ok_or_else(|| {
        Error::new(
            ErrorKind::NotFound,
            format!("Could not resolve bind address: {address}"),
        )
    })
}

fn is_ipv4_wildcard(address: SocketAddr) -> bool {
    address.ip() == IpAddr::V4(Ipv4Addr::UNSPECIFIED)
}

/// Creates a non-blocking IPv6 socket that also accepts IPv4 traffic
/// (IPv4 peers show up as `::ffff:a.b.c.d`).
fn new_dual_stack_socket(socket_type: Type, protocol: Protocol, port: u16) -> Result<Socket> {
    let socket = Socket::new(Domain::IPV6, socket_type, Some(protocol))?;
    socket.set_only_v6(false)?;
    socket.set_reuse_address(true)?;
    socket.set_nonblocking(true)?;
    socket.bind(&SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), port).into())?;

    Ok(socket)
}

/// Binds a TCP listener on `address:port`.
///
/// The IPv4 wildcard (`0.0.0.0`, the default everywhere) binds a dual-stack
/// socket so that both IPv4 and IPv6 clients can connect. Hosts without IPv6
/// support fall back to an IPv4-only socket, so this never makes binding fail
/// where it used to succeed.
pub async fn bind_tcp_listener(address: &str, port: u16) -> Result<TcpListener> {
    let bind_address = resolve_bind_address(address, port).await?;

    if is_ipv4_wildcard(bind_address) {
        match new_dual_stack_socket(Type::STREAM, Protocol::TCP, port) {
            Ok(socket) => {
                socket.listen(1024)?;
                return TcpListener::from_std(socket.into());
            }
            Err(e) => debug!("Dual-stack bind on port {port} failed ({e}), using IPv4 only"),
        }
    }

    TcpListener::bind(bind_address).await
}

/// Binds a UDP socket on `address:port` with the same dual-stack behaviour as
/// [`bind_tcp_listener`].
pub async fn bind_udp_socket(address: &str, port: u16) -> Result<UdpSocket> {
    let bind_address = resolve_bind_address(address, port).await?;

    if is_ipv4_wildcard(bind_address) {
        match new_dual_stack_socket(Type::DGRAM, Protocol::UDP, port) {
            Ok(socket) => return UdpSocket::from_std(socket.into()),
            Err(e) => debug!("Dual-stack bind on port {port} failed ({e}), using IPv4 only"),
        }
    }

    UdpSocket::bind(bind_address).await
}

#[cfg(test)]
mod tests {
    use tokio::net::TcpStream;

    use super::*;

    #[tokio::test]
    async fn test_bind_tcp_listener_resolves_literals_and_names() {
        assert_eq!(
            resolve_bind_address("127.0.0.1", 80).await.unwrap(),
            "127.0.0.1:80".parse().unwrap()
        );
        assert_eq!(
            resolve_bind_address("[::1]", 80).await.unwrap(),
            "[::1]:80".parse().unwrap()
        );
        assert!(resolve_bind_address("localhost", 80).await.is_ok());
    }

    #[tokio::test]
    async fn test_bind_tcp_listener_wildcard_accepts_ipv4_and_ipv6() {
        let listener = bind_tcp_listener("0.0.0.0", 0).await.unwrap();
        let port = listener.local_addr().unwrap().port();

        TcpStream::connect(("127.0.0.1", port)).await.unwrap();
        let (_, peer) = listener.accept().await.unwrap();
        assert!(peer.ip().is_loopback() || peer.ip().to_canonical().is_loopback());

        if listener.local_addr().unwrap().is_ipv6() {
            TcpStream::connect(("::1", port)).await.unwrap();
            listener.accept().await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_bind_tcp_listener_specific_address() {
        let listener = bind_tcp_listener("127.0.0.1", 0).await.unwrap();
        assert!(listener.local_addr().unwrap().ip().is_loopback());
    }

    #[tokio::test]
    async fn test_bind_udp_socket_wildcard_receives_ipv4() {
        let server = bind_udp_socket("0.0.0.0", 0).await.unwrap();
        let port = server.local_addr().unwrap().port();

        let client = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        client.send_to(b"ping", ("127.0.0.1", port)).await.unwrap();

        let mut buffer = [0u8; 8];
        let (size, peer) = server.recv_from(&mut buffer).await.unwrap();
        assert_eq!(&buffer[..size], b"ping");

        server.send_to(b"pong", peer).await.unwrap();
        let (size, _) = client.recv_from(&mut buffer).await.unwrap();
        assert_eq!(&buffer[..size], b"pong");
    }
}
