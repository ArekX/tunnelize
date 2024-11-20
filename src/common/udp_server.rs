use std::{collections::HashMap, net::SocketAddr, time::Instant};

use super::{
    channel_socket::{ChannelPacket, ChannelSocket},
    connection::Connection,
};
use bytes::BytesMut;
use log::error;
use tokio::{
    io::Result,
    net::UdpSocket,
    sync::mpsc::{channel, Receiver, Sender},
};
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

pub struct ReceivedClient {
    pub id: Uuid,
    pub connection: Connection,
    pub data: Vec<u8>,
}

pub struct UdpServer {
    socket: UdpSocket,
    data_buffer: BytesMut,
    cancel_token: CancellationToken,
    max_timeout: u64,
    adress_to_tx_map: HashMap<SocketAddr, ActiveClient>,
    server_tx: Sender<ChannelPacket>,
}

impl UdpServer {
    pub async fn new(
        port: u16,
        address: Option<String>,
        max_timeout: u64,
        cancel_token: CancellationToken,
    ) -> Result<(Self, Receiver<ChannelPacket>)> {
        let address_port = format!(
            "{}:{}",
            address.unwrap_or_else(|| "0.0.0.0".to_string()),
            port
        );

        let (server_tx, server_rx) = channel::<ChannelPacket>(100);

        let mut data_buffer = BytesMut::with_capacity(2048);
        data_buffer.resize(2048, 0);

        Ok((
            Self {
                socket: UdpSocket::bind(address_port).await?,
                data_buffer,
                cancel_token,
                max_timeout,
                adress_to_tx_map: HashMap::new(),
                server_tx,
            },
            server_rx,
        ))
    }

    pub async fn handle_channel_packet(&mut self, packet: ChannelPacket) {
        match packet {
            ChannelPacket::Data(address, data) => {
                if let Err(e) = self.write(&data, &address).await {
                    error!("Failed to send data to client: {}", e);
                }
            }
            ChannelPacket::Shutdown(address) => {
                self.adress_to_tx_map.remove(&address);
            }
        }
    }

    pub async fn listen_for_connections(&mut self) -> Result<ReceivedClient> {
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
                ChannelSocket::new(address, self.server_tx.clone(), cancel_token.clone());

            let socket_tx = channel_socket.get_socket_tx();

            let active_client = ActiveClient {
                socket_tx,
                cancel_token,
                last_activity: Instant::now(),
            };

            let client_id = Uuid::new_v4();
            self.adress_to_tx_map.insert(address, active_client);

            return Ok(ReceivedClient {
                id: client_id,
                connection: Connection::from(channel_socket),
                data,
            });
        }
    }

    pub async fn write(&mut self, data: &[u8], address: &SocketAddr) -> Result<usize> {
        match self.socket.send_to(data, address).await {
            Ok(size) => {
                if let Some(client) = self.adress_to_tx_map.get_mut(address) {
                    client.update_activity();
                }
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

    pub fn shutdown(&mut self) {
        self.cancel_token.cancel();
        for (_, mut client) in self.adress_to_tx_map.drain() {
            client.close();
        }
    }
}
