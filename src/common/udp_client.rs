use std::io::{Error, ErrorKind};
use std::net::{SocketAddr, ToSocketAddrs};

use tokio::io::Result;
use tokio::net::UdpSocket;
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct UdpClient {
    listener_socket: UdpSocket,
    host: String,
    port: u16,
    destination_address: Option<SocketAddr>,
    cancel_token: CancellationToken,
}

impl UdpClient {
    pub async fn new(
        host: String,
        port: u16,
        cancel_token: CancellationToken,
        bind_address: Option<String>,
    ) -> Result<Self> {
        let listener_socket =
            UdpSocket::bind(bind_address.clone().unwrap_or("0.0.0.0:0".to_string())).await?;

        Ok(Self {
            listener_socket,
            host,
            port,
            cancel_token,
            destination_address: None,
        })
    }

    pub async fn read(&mut self, buffer: &mut [u8]) -> Result<usize> {
        let (size, _) = self.listener_socket.recv_from(buffer).await?;

        Ok(size)
    }

    pub async fn write(&mut self, buffer: &[u8]) -> Result<usize> {
        if let Some(destination_address) = self.destination_address {
            return self
                .listener_socket
                .send_to(buffer, destination_address)
                .await;
        }

        let addr_port = format!("{}:{}", self.host, self.port);
        let socket_addrs: Vec<SocketAddr> = addr_port.to_socket_addrs()?.collect();

        if socket_addrs.is_empty() {
            return Err(Error::new(
                ErrorKind::NotFound,
                format!("Could not resolve address: {}", addr_port),
            ));
        }

        let mut error = None;

        for addr in socket_addrs.iter().filter(|addr| addr.is_ipv6()) {
            match self.listener_socket.send_to(buffer, addr).await {
                Ok(size) => {
                    self.destination_address = Some(addr.clone());
                    return Ok(size);
                }
                Err(e) => error = Some(e),
            }
        }

        for addr in socket_addrs.iter().filter(|addr| addr.is_ipv4()) {
            match self.listener_socket.send_to(buffer, addr).await {
                Ok(size) => {
                    self.destination_address = Some(addr.clone());
                    return Ok(size);
                }
                Err(e) => error = Some(e),
            }
        }

        if let Some(e) = error {
            return Err(e);
        };

        Err(Error::new(
            ErrorKind::Other,
            format!("Failed to connect to address: {}", addr_port),
        ))
    }

    pub fn get_cancel_token(&self) -> CancellationToken {
        self.cancel_token.clone()
    }

    pub fn shutdown(&self) {
        self.cancel_token.cancel();
    }
}
