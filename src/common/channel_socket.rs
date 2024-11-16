use std::io::{Error, ErrorKind};

use tokio::io::Result;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub struct ChannelPacket(pub Uuid, pub Vec<u8>);

#[derive(Debug)]
pub struct ChannelSocket {
    id: Uuid,
    pub link_to_tx: Sender<ChannelPacket>,
    pub socket_rx: Receiver<Vec<u8>>,
    pub socket_tx: Sender<Vec<u8>>,
    cancel_token: CancellationToken,
}

impl ChannelSocket {
    pub fn new(link_to_tx: Sender<ChannelPacket>, cancel_token: CancellationToken) -> Self {
        let (socket_tx, socket_rx) = mpsc::channel(1);

        Self {
            id: Uuid::new_v4(),
            link_to_tx,
            socket_rx,
            socket_tx,
            cancel_token,
        }
    }

    pub fn get_id(&self) -> Uuid {
        self.id
    }

    pub fn get_socket_tx(&self) -> Sender<Vec<u8>> {
        self.socket_tx.clone()
    }

    pub async fn send(&self, data: Vec<u8>) -> Result<()> {
        match self.link_to_tx.send(ChannelPacket(self.id, data)).await {
            Ok(_) => Ok(()),
            Err(_) => Err(tokio::io::Error::new(
                tokio::io::ErrorKind::Other,
                "Failed to send data to link",
            )),
        }
    }

    pub async fn receive(&mut self) -> tokio::io::Result<Vec<u8>> {
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
                self.shutdown();
                Err(Error::new(
                    ErrorKind::ConnectionAborted,
                    "Failed to receive data from link",
                ))
            }
        }
    }

    pub fn shutdown(&mut self) {
        self.socket_rx.close();
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
        let socket = ChannelSocket::new(tx, cancel_token);

        assert_eq!(socket.socket_rx.capacity(), 1);
        assert_eq!(socket.socket_tx.capacity(), 1);
    }

    #[tokio::test]
    async fn test_channel_socket_get_id() {
        let (tx, _rx) = mpsc::channel(1);
        let cancel_token = CancellationToken::new();
        let socket = ChannelSocket::new(tx, cancel_token);

        assert!(!socket.get_id().is_nil());
    }

    #[tokio::test]
    async fn test_channel_socket_send() {
        let (tx, mut rx) = mpsc::channel(1);
        let cancel_token = CancellationToken::new();
        let socket = ChannelSocket::new(tx, cancel_token);

        let data = vec![1, 2, 3];
        socket.send(data.clone()).await.unwrap();

        let received = rx.recv().await.unwrap();
        assert_eq!(received.1, data);
    }

    #[tokio::test]
    async fn test_channel_socket_receive() {
        let (tx, _rx) = mpsc::channel(1);
        let cancel_token = CancellationToken::new();
        let mut socket = ChannelSocket::new(tx, cancel_token);

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
        let mut socket = ChannelSocket::new(tx, cancel_token.clone());

        cancel_token.cancel();
        let result = socket.receive().await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().kind(), ErrorKind::ConnectionAborted);
    }

    #[tokio::test]
    async fn test_top_channel_receives_written_data() {
        let (tx, mut rx) = mpsc::channel(1);
        let cancel_token = CancellationToken::new();
        let socket = ChannelSocket::new(tx, cancel_token);

        let data = vec![1, 2, 3];
        socket.send(data.clone()).await.unwrap();

        let received = rx.recv().await.unwrap();
        assert_eq!(received.0, socket.get_id());
        assert_eq!(received.1, data);
    }

    #[tokio::test]
    async fn test_channel_socket_shutdown() {
        let (tx, _rx) = mpsc::channel(1);
        let cancel_token = CancellationToken::new();
        let mut socket = ChannelSocket::new(tx, cancel_token);

        socket.shutdown();
        assert!(socket.socket_rx.is_closed());
    }
}
