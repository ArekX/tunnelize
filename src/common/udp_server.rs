use std::{collections::HashMap, net::SocketAddr, time::Instant};

use super::{
    channel_socket::{ChannelPacket, ChannelSocket},
    connection::Connection,
};
use bytes::BytesMut;
use log::error;
use tokio::{io::Result, net::UdpSocket, sync::mpsc::Sender};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

type DataSender = Sender<Vec<u8>>;

pub struct ActiveClient {
    pub socket_tx: DataSender,
    pub cancel_token: CancellationToken,
    last_activity: Instant,
}

impl ActiveClient {
    pub fn is_active(&self, max_timeout: u64) -> bool {
        !self.is_expired(max_timeout)
            && !self.socket_tx.is_closed()
            && !self.cancel_token.is_cancelled()
    }

    pub fn is_expired(&self, max_timeout: u64) -> bool {
        self.last_activity.elapsed().as_secs() >= max_timeout
    }

    pub fn update_activity(&mut self) {
        self.last_activity = Instant::now();
    }

    pub fn close(&mut self) {
        self.cancel_token.cancel();
    }
}

pub struct Client {
    pub id: Uuid,
    pub connection: Connection,
    pub address: SocketAddr,
    pub data: Vec<u8>,
}

pub struct UdpServer {
    socket: UdpSocket,
    data_buffer: BytesMut,
    data_reciver_tx: Sender<ChannelPacket>,
    cancel_token: CancellationToken,
    max_timeout: u64,
    // TODO: Use Rc<ActiveClient> instead of ActiveClient, to match by ID and address
    // TODO: Udp server should have its own TX/RX, channel socket
    adress_to_tx_map: HashMap<SocketAddr, ActiveClient>,
}

impl UdpServer {
    pub async fn new(
        port: u16,
        address: Option<String>,
        data_reciver_tx: Sender<ChannelPacket>,
        max_timeout: u64,
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
            max_timeout,
            adress_to_tx_map: HashMap::new(),
        })
    }

    pub async fn listen_for_connections(&mut self) -> Result<Client> {
        loop {
            let (data, address) = match self.socket.recv_from(&mut self.data_buffer).await {
                Ok((size, address)) => (self.data_buffer[..size].to_vec(), address),
                Err(e) => {
                    error!("Failed to read data from client: {}", e);
                    return Err(e);
                }
            };

            if let Some(client) = self.adress_to_tx_map.get_mut(&address) {
                if client.is_active(self.max_timeout) {
                    if let Err(e) = client.socket_tx.send(data).await {
                        error!("Failed to send data to client: {}", e);
                    } else {
                        client.update_activity();
                    }
                    continue;
                }

                client.close();
                self.adress_to_tx_map.remove(&address);
            }

            let cancel_token = self.cancel_token.child_token();

            let channel_socket =
                ChannelSocket::new(self.data_reciver_tx.clone(), cancel_token.clone());

            let socket_tx = channel_socket.get_socket_tx();

            self.adress_to_tx_map.insert(
                address,
                ActiveClient {
                    socket_tx: socket_tx.clone(),
                    cancel_token,
                    last_activity: Instant::now(),
                },
            );

            return Ok(Client {
                id: channel_socket.get_id(),
                connection: Connection::from(channel_socket),
                address,
                data,
            });
        }
    }

    pub async fn write(&mut self, data: &[u8], address: &SocketAddr) -> Result<usize> {
        match self.socket.send_to(data, address).await {
            Ok(size) => {
                self.adress_to_tx_map
                    .get_mut(address)
                    .map(|client| client.update_activity());
                Ok(size)
            }
            Err(e) => Err(e),
        }
    }

    pub fn cleanup_inactive_clients(&mut self) {
        self.adress_to_tx_map.retain(|_, client| {
            if !client.is_active(self.max_timeout) {
                client.close();
                return false;
            }

            true
        });
    }
}
