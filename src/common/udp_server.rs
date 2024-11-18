use std::net::SocketAddr;

use super::{
    channel_socket::{ChannelPacket, ChannelSocket},
    connection::Connection,
};
use bytes::BytesMut;
use log::error;
use tokio::{io::Result, net::UdpSocket, sync::mpsc::Sender};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub struct Client {
    pub id: Uuid,
    pub connection: Connection,
    pub address: SocketAddr,
    pub socket_tx: Sender<Vec<u8>>,
    pub data: Vec<u8>,
}

pub struct UdpServer {
    socket: UdpSocket,
    data_buffer: BytesMut,
    data_reciver_tx: Sender<ChannelPacket>,
    cancel_token: CancellationToken,
}

impl UdpServer {
    pub async fn new(
        port: u16,
        address: Option<String>,
        data_reciver_tx: Sender<ChannelPacket>,
        cancel_token: CancellationToken,
    ) -> Result<Self> {
        let address_port = format!(
            "{}:{}",
            address.unwrap_or_else(|| "0.0.0.0".to_string()),
            port
        );

        let mut data_buffer = BytesMut::with_capacity(2048);
        data_buffer.resize(2048, 0);

        Ok(Self {
            socket: UdpSocket::bind(address_port).await?,
            data_buffer,
            data_reciver_tx,
            cancel_token,
        })
    }

    pub async fn listen_for_connections(&mut self) -> Result<Client> {
        let (initial_data, address) = match self.socket.recv_from(&mut self.data_buffer).await {
            Ok((size, address)) => (self.data_buffer[..size].to_vec(), address),
            Err(e) => {
                error!("Failed to read data from client: {}", e);
                return Err(e);
            }
        };

        // TODO: Keep track of address, socket_tx. when existing client is found, send data to client via socket_tx. this will avoid overhead of creating new client for each packet.

        let channel_socket = ChannelSocket::new(
            self.data_reciver_tx.clone(),
            self.cancel_token.child_token(),
        );

        let socket_tx = channel_socket.get_socket_tx();
        Ok(Client {
            id: channel_socket.get_id(),
            connection: Connection::from(channel_socket),
            address,
            data: initial_data,
            socket_tx,
        })
    }

    pub async fn write(&self, data: &[u8], address: &SocketAddr) -> Result<usize> {
        self.socket.send_to(data, address).await
    }
}
