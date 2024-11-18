use std::io::ErrorKind;

use bytes::BytesMut;
use log::{debug, error};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, Result},
    net::TcpStream,
};
use tokio_rustls::{client::TlsStream as ClientTlsStream, server::TlsStream as ServerTlsStream};

use super::{channel_socket::ChannelSocket, udp_client::UdpClient};

pub trait DataBridge<To> {
    async fn bridge_to(&mut self, to: &mut To) -> Result<()>;
}

impl DataBridge<TcpStream> for TcpStream {
    async fn bridge_to(&mut self, to: &mut TcpStream) -> Result<()> {
        match tokio::io::copy_bidirectional(self, to).await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Failed to bridge data: {}", e);
                Err(e)
            }
        }
    }
}

impl DataBridge<ServerTlsStream<TcpStream>> for TcpStream {
    async fn bridge_to(&mut self, to: &mut ServerTlsStream<TcpStream>) -> Result<()> {
        match tokio::io::copy_bidirectional(self, to).await {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => {
                debug!("Server TLS connection ended: {:?}", e);
                Ok(())
            }
            Err(e) => {
                error!("Failed to bridge data: {}", e);
                Err(e)
            }
        }
    }
}

impl DataBridge<ServerTlsStream<TcpStream>> for ServerTlsStream<TcpStream> {
    async fn bridge_to(&mut self, to: &mut ServerTlsStream<TcpStream>) -> Result<()> {
        match tokio::io::copy_bidirectional(self, to).await {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => {
                debug!("Server TLS connection ended: {:?}", e);
                Ok(())
            }
            Err(e) => {
                error!("Failed to bridge data: {}", e);
                Err(e)
            }
        }
    }
}

impl DataBridge<ClientTlsStream<TcpStream>> for TcpStream {
    async fn bridge_to(&mut self, to: &mut ClientTlsStream<TcpStream>) -> Result<()> {
        match tokio::io::copy_bidirectional(self, to).await {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == ErrorKind::UnexpectedEof => {
                debug!("Client TLS connection ended: {:?}", e);
                Ok(())
            }
            Err(e) => {
                error!("Failed to bridge data: {}", e);
                Err(e)
            }
        }
    }
}

impl DataBridge<TcpStream> for ServerTlsStream<TcpStream> {
    async fn bridge_to(&mut self, to: &mut TcpStream) -> Result<()> {
        match tokio::io::copy_bidirectional(self, to).await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Failed to bridge data: {}", e);
                Err(e)
            }
        }
    }
}

impl DataBridge<UdpClient> for TcpStream {
    async fn bridge_to(&mut self, to: &mut UdpClient) -> Result<()> {
        bridge_udp_with_writable(to, self).await
    }
}

impl DataBridge<UdpClient> for ServerTlsStream<TcpStream> {
    async fn bridge_to(&mut self, to: &mut UdpClient) -> Result<()> {
        bridge_udp_with_writable(to, self).await
    }
}

impl DataBridge<TcpStream> for UdpClient {
    async fn bridge_to(&mut self, to: &mut TcpStream) -> Result<()> {
        bridge_udp_with_writable(self, to).await
    }
}

impl DataBridge<ServerTlsStream<TcpStream>> for UdpClient {
    async fn bridge_to(&mut self, to: &mut ServerTlsStream<TcpStream>) -> Result<()> {
        bridge_udp_with_writable(self, to).await
    }
}

impl DataBridge<ClientTlsStream<TcpStream>> for UdpClient {
    async fn bridge_to(&mut self, to: &mut ClientTlsStream<TcpStream>) -> Result<()> {
        bridge_udp_with_writable(self, to).await
    }
}

async fn bridge_udp_with_writable<T: AsyncWriteExt + Unpin + AsyncReadExt>(
    udp_client: &mut UdpClient,
    writable: &mut T,
) -> Result<()> {
    let mut udp_buffer = BytesMut::with_capacity(2048);
    udp_buffer.resize(2048, 0);

    let mut writable_buffer = BytesMut::with_capacity(2048);
    writable_buffer.resize(2048, 0);

    let cancel_token = udp_client.get_cancel_token();

    loop {
        tokio::select! {
            _ = cancel_token.cancelled() => {
                break;
            }
            result = udp_client.read(&mut udp_buffer) => {
                match result {
                    Ok(n) => {
                        if let Err(e) = writable.write_all(&udp_buffer[..n]).await {
                            error!("Failed to send data to Writable stream: {}", e);
                        }
                    }
                    Err(e)
                        if e.kind() == std::io::ErrorKind::UnexpectedEof
                            || e.kind() == std::io::ErrorKind::ConnectionAborted
                            || e.kind() == std::io::ErrorKind::ConnectionReset
                            || e.kind() == std::io::ErrorKind::BrokenPipe =>
                    {
                        debug!("Writable <-> UDP connection ended: {:?}", e);
                        cancel_token.cancel();
                        break;
                    }
                    Err(e) => {
                        error!("Failed to receive data from UDP socket: {}", e);
                    }
                }
            }
            result = writable.read(&mut writable_buffer) => {
                match result {
                    Ok(0) => {
                        cancel_token.cancel();
                        break;
                    },
                    Ok(n) => {
                        if let Err(e) = udp_client.write(&writable_buffer[..n]).await {
                            error!("Failed to send data to UDP socket: {}", e);
                        }
                    }
                    Err(e)
                        if e.kind() == std::io::ErrorKind::UnexpectedEof
                            || e.kind() == std::io::ErrorKind::ConnectionAborted
                            || e.kind() == std::io::ErrorKind::ConnectionReset
                            || e.kind() == std::io::ErrorKind::BrokenPipe =>
                    {
                        debug!("Writable <-> UDP connection ended: {:?}", e);
                        cancel_token.cancel();
                        break;
                    }
                    Err(e) => {
                        error!("Failed to receive data from Writable stream: {}", e);
                    }
                }
            }
        }
    }

    Ok(())
}

impl DataBridge<ChannelSocket> for TcpStream {
    async fn bridge_to(&mut self, to: &mut ChannelSocket) -> Result<()> {
        bridge_channel_socket_with_writable(to, self).await
    }
}

impl DataBridge<ChannelSocket> for ServerTlsStream<TcpStream> {
    async fn bridge_to(&mut self, to: &mut ChannelSocket) -> Result<()> {
        bridge_channel_socket_with_writable(to, self).await
    }
}

async fn bridge_channel_socket_with_writable<T: AsyncWriteExt + Unpin + AsyncReadExt>(
    channel_socket: &mut ChannelSocket,
    writable: &mut T,
) -> Result<()> {
    let mut writable_buffer = BytesMut::with_capacity(2048);
    writable_buffer.resize(2048, 0);

    loop {
        tokio::select! {
            result = channel_socket.receive() => {
                match result {
                    Ok(bytes) => {
                        if let Err(e) = writable.write_all(&bytes).await {
                            error!("Failed to send data to Writable stream: {}", e);
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::ConnectionAborted =>
                    {
                        debug!("Writable <-> Channel socket connection ended: {:?}", e);
                        break;
                    }
                    Err(e) => {
                        error!("Failed to receive data from Channel socket: {}", e);
                    }
                }
            }
            result = writable.read(&mut writable_buffer) => {
                match result {
                    Ok(0) => {
                        channel_socket.shutdown();
                        break;
                    },
                    Ok(_) => {
                        if let Err(e) = channel_socket.send(writable_buffer.to_vec()).await {
                            error!("Failed to send data to Channel socket: {}", e);
                        }
                    }
                    Err(e)
                        if e.kind() == std::io::ErrorKind::UnexpectedEof
                            || e.kind() == std::io::ErrorKind::ConnectionAborted
                            || e.kind() == std::io::ErrorKind::ConnectionReset
                            || e.kind() == std::io::ErrorKind::BrokenPipe =>
                    {
                        debug!("Writable <-> Channel Socket connection ended: {:?}", e);
                        channel_socket.shutdown();
                        break;
                    }
                    Err(e) => {
                        error!("Failed to receive data from Writable stream: {}", e);
                    }
                }
            }
        }
    }

    Ok(())
}
