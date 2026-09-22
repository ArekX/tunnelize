use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

use tokio::io::Result;
use tokio::net::UdpSocket;
use tokio_util::sync::CancellationToken;

use super::protocol_socket::resolve_address;

#[derive(Debug)]
pub struct UdpClient {
    listener_socket: UdpSocket,
    destination_address: SocketAddr,
    cancel_token: CancellationToken,
}

impl UdpClient {
    pub async fn new(
        host: String,
        port: u16,
        cancel_token: CancellationToken,
        bind_address: Option<String>,
    ) -> Result<Self> {
        // Resolve up front (async, bounded) so that sending never blocks the
        // runtime on DNS, and so the local socket family matches the destination.
        let destination_address = resolve_address(&host, port).await?[0];

        let listener_socket = match bind_address {
            Some(address) => match address
                .trim_start_matches('[')
                .trim_end_matches(']')
                .parse::<IpAddr>()
            {
                Ok(ip) => UdpSocket::bind(SocketAddr::new(ip, 0)).await?,
                Err(_) => UdpSocket::bind(address).await?,
            },
            None => {
                let wildcard = match destination_address {
                    SocketAddr::V4(_) => IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                    SocketAddr::V6(_) => IpAddr::V6(Ipv6Addr::UNSPECIFIED),
                };
                UdpSocket::bind(SocketAddr::new(wildcard, 0)).await?
            }
        };

        Ok(Self {
            listener_socket,
            destination_address,
            cancel_token,
        })
    }

    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<usize> {
        let (size, _) = self.listener_socket.recv_from(buffer).await?;

        Ok(size)
    }

    pub async fn write(&mut self, buffer: &[u8]) -> Result<usize> {
        self.listener_socket
            .send_to(buffer, self.destination_address)
            .await
    }

    pub fn get_cancel_token(&self) -> CancellationToken {
        self.cancel_token.clone()
    }

    pub fn shutdown(&self) {
        self.cancel_token.cancel();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_udp_client_round_trip() {
        let server = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let port = server.local_addr().unwrap().port();

        let mut client = UdpClient::new(
            "localhost".to_string(),
            port,
            CancellationToken::new(),
            None,
        )
        .await
        .unwrap();

        assert!(client.destination_address.is_ipv4());

        client.write(b"ping").await.unwrap();

        let mut buffer = [0u8; 8];
        let (size, peer) = server.recv_from(&mut buffer).await.unwrap();
        assert_eq!(&buffer[..size], b"ping");

        server.send_to(b"pong", peer).await.unwrap();
        let size = client.read(&mut buffer).await.unwrap();
        assert_eq!(&buffer[..size], b"pong");
    }

    #[tokio::test]
    async fn test_udp_client_with_bind_address() {
        let client = UdpClient::new(
            "127.0.0.1".to_string(),
            9,
            CancellationToken::new(),
            Some("127.0.0.1".to_string()),
        )
        .await
        .unwrap();

        assert!(
            client
                .listener_socket
                .local_addr()
                .unwrap()
                .ip()
                .is_loopback()
        );
    }
}
