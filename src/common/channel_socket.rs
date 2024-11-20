use std::io::{Error, ErrorKind};
use std::net::SocketAddr;

use log::error;
use tokio::io::Result;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio_util::sync::CancellationToken;

pub enum ChannelPacket {
    Data(SocketAddr, Vec<u8>),
    Shutdown(SocketAddr),
}

#[derive(Debug)]
pub struct ChannelSocket {
    address: SocketAddr,
    pub link_to_tx: Sender<ChannelPacket>,
    pub socket_rx: Receiver<Vec<u8>>,
    pub socket_tx: Sender<Vec<u8>>,
    cancel_token: CancellationToken,
}

impl ChannelSocket {
    pub fn new(
        address: SocketAddr,
        link_to_tx: Sender<ChannelPacket>,
        cancel_token: CancellationToken,
    ) -> Self {
        let (socket_tx, socket_rx) = mpsc::channel(1);

        Self {
            address,
            link_to_tx,
            socket_rx,
            socket_tx,
            cancel_token,
        }
    }

    pub fn get_address(&self) -> SocketAddr {
        self.address
    }

    pub fn get_socket_tx(&self) -> Sender<Vec<u8>> {
        self.socket_tx.clone()
    }

    pub async fn send(&self, data: Vec<u8>) -> Result<()> {
        match self
            .link_to_tx
            .send(ChannelPacket::Data(self.address, data))
            .await
        {
            Ok(_) => Ok(()),
            Err(_) => Err(tokio::io::Error::new(
                tokio::io::ErrorKind::Other,
                "Failed to send data to link",
            )),
        }
    }

    pub async fn receive(&mut self) -> Result<Vec<u8>> {
        tokio::select! {
            data = self.socket_rx.recv() => {
                match data {
                    Some(data) => Ok(data),
                    None => Err(Error::new(
                        ErrorKind::ConnectionAborted,
                        "Failed to receive data from link",
                    )),
                }
            }
            _ = self.cancel_token.cancelled() => {
                self.shutdown().await;
                Err(Error::new(
                    ErrorKind::ConnectionAborted,
                    "Failed to receive data from link",
                ))
            }
        }
    }

    pub async fn shutdown(&mut self) {
        if let Err(e) = self
            .link_to_tx
            .send(ChannelPacket::Shutdown(self.address))
            .await
        {
            error!("Failed to send shutdown message: {}", e);
        }
        self.socket_rx.close();
        self.cancel_token.cancel();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;
    use tokio_util::sync::CancellationToken;

    #[tokio::test]
    async fn test_channel_socket_new() {
        let (tx, _rx) = mpsc::channel(1);
        let cancel_token = CancellationToken::new();
        let address = "127.0.0.1:8080".parse().unwrap();
        let socket = ChannelSocket::new(address, tx, cancel_token);

        assert_eq!(socket.socket_rx.capacity(), 1);
        assert_eq!(socket.socket_tx.capacity(), 1);
    }

    #[tokio::test]
    async fn test_channel_socket_get_address() {
        let (tx, _rx) = mpsc::channel(1);
        let cancel_token = CancellationToken::new();
        let address = "127.0.0.1:8080".parse().unwrap();
        let socket = ChannelSocket::new(address, tx, cancel_token);

        assert_eq!(socket.get_address(), address);
    }

    #[tokio::test]
    async fn test_channel_socket_send() {
        let (tx, mut rx) = mpsc::channel(1);
        let cancel_token = CancellationToken::new();
        let address = "127.0.0.1:8080".parse().unwrap();
        let socket = ChannelSocket::new(address, tx, cancel_token);

        let data = vec![1, 2, 3];
        socket.send(data.clone()).await.unwrap();

        let received = rx.recv().await.unwrap();
        if let ChannelPacket::Data(addr, received_data) = received {
            assert_eq!(addr, address);
            assert_eq!(received_data, data);
        } else {
            panic!("Expected ChannelPacket::Data");
        }
    }

    #[tokio::test]
    async fn test_channel_socket_receive() {
        let (tx, _rx) = mpsc::channel(1);
        let cancel_token = CancellationToken::new();
        let address = "127.0.0.1:8080".parse().unwrap();
        let mut socket = ChannelSocket::new(address, tx, cancel_token);

        let data = vec![1, 2, 3];

        let socket_tx = socket.get_socket_tx();

        socket_tx.send(data.clone()).await.unwrap();

        let received = socket.receive().await.unwrap();
        assert_eq!(received, data);
    }

    #[tokio::test]
    async fn test_channel_socket_receive_cancelled() {
        let (tx, _rx) = mpsc::channel(1);
        let cancel_token = CancellationToken::new();
        let address = "127.0.0.1:8080".parse().unwrap();
        let mut socket = ChannelSocket::new(address, tx, cancel_token.clone());

        cancel_token.cancel();
        let result = socket.receive().await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::ConnectionAborted);
    }

    #[tokio::test]
    async fn test_top_channel_receives_written_data() {
        let (tx, mut rx) = mpsc::channel(1);
        let cancel_token = CancellationToken::new();
        let address = "127.0.0.1:8080".parse().unwrap();
        let socket = ChannelSocket::new(address, tx, cancel_token);

        let data = vec![1, 2, 3];
        socket.send(data.clone()).await.unwrap();

        let received = rx.recv().await.unwrap();
        if let ChannelPacket::Data(addr, received_data) = received {
            assert_eq!(addr, socket.get_address());
            assert_eq!(received_data, data);
        } else {
            panic!("Expected ChannelPacket::Data");
        }
    }

    #[tokio::test]
    async fn test_channel_socket_shutdown() {
        let (tx, _rx) = mpsc::channel(1);
        let cancel_token = CancellationToken::new();
        let address = "127.0.0.1:8080".parse().unwrap();
        let mut socket = ChannelSocket::new(address, tx, cancel_token);

        socket.shutdown().await;
        assert!(socket.socket_rx.is_closed());
    }
}
