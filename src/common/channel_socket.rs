use tokio::io::Result;
use tokio::sync::mpsc::{self, Receiver, Sender};

#[derive(Debug)]
pub struct ChannelSocket {
    pub link_to_tx: Sender<Vec<u8>>,
    pub socket_rx: Receiver<Vec<u8>>,
    pub socket_tx: Sender<Vec<u8>>,
}

impl ChannelSocket {
    pub fn new(link_to_tx: Sender<Vec<u8>>) -> Self {
        let (socket_tx, socket_rx) = mpsc::channel(1);

        Self {
            link_to_tx,
            socket_rx,
            socket_tx,
        }
    }

    pub fn get_socket_tx(&self) -> Sender<Vec<u8>> {
        self.socket_tx.clone()
    }

    pub async fn send(&self, data: Vec<u8>) -> Result<()> {
        match self.link_to_tx.send(data).await {
            Ok(_) => Ok(()),
            Err(_) => Err(tokio::io::Error::new(
                tokio::io::ErrorKind::Other,
                "Failed to send data to link",
            )),
        }
    }

    pub async fn receive(&mut self) -> Vec<u8> {
        match self.socket_rx.recv().await {
            Some(data) => data,
            None => Vec::new(),
        }
    }

    pub fn shutdown(&mut self) {
        self.socket_rx.close();
    }

    // TODO: Implement a method to close the socket
    // TODO: Implement similar methods to sockets like read and write.
}
