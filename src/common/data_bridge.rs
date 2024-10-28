use std::{
    io::{Error, ErrorKind},
    net::SocketAddr,
};

use bytes::BytesMut;
use log::{debug, error};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, Result},
    net::{TcpStream, UdpSocket},
};
use tokio_rustls::server::TlsStream as ServerTlsStream;
use tokio_util::sync::CancellationToken;

use super::channel_socket::ChannelSocket;

pub trait DataBridge<To> {
    type Context;
    async fn bridge_to(&mut self, to: &mut To, context: Option<Self::Context>) -> Result<()>;
}

impl DataBridge<TcpStream> for TcpStream {
    type Context = ();
    async fn bridge_to(
        &mut self,
        to: &mut TcpStream,
        _context: Option<Self::Context>,
    ) -> Result<()> {
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
    type Context = ();
    async fn bridge_to(
        &mut self,
        to: &mut ServerTlsStream<TcpStream>,
        _context: Option<Self::Context>,
    ) -> Result<()> {
        match tokio::io::copy_bidirectional(self, to).await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Failed to bridge data: {}", e);
                Err(e)
            }
        }
    }
}

impl DataBridge<TcpStream> for ServerTlsStream<TcpStream> {
    type Context = ();
    async fn bridge_to(
        &mut self,
        to: &mut TcpStream,
        _context: Option<Self::Context>,
    ) -> Result<()> {
        match tokio::io::copy_bidirectional(self, to).await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Failed to bridge data: {}", e);
                Err(e)
            }
        }
    }
}

impl DataBridge<ServerTlsStream<TcpStream>> for ServerTlsStream<TcpStream> {
    type Context = ();
    async fn bridge_to(
        &mut self,
        to: &mut ServerTlsStream<TcpStream>,
        _context: Option<Self::Context>,
    ) -> Result<()> {
        match tokio::io::copy_bidirectional(self, to).await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Failed to bridge data: {}", e);
                Err(e)
            }
        }
    }
}

impl DataBridge<UdpSocket> for TcpStream {
    type Context = UdpSession;
    async fn bridge_to(
        &mut self,
        to: &mut UdpSocket,
        context: Option<Self::Context>,
    ) -> Result<()> {
        bridge_udp_with_writable(to, self, context).await
    }
}

impl DataBridge<UdpSocket> for ServerTlsStream<TcpStream> {
    type Context = UdpSession;
    async fn bridge_to(
        &mut self,
        to: &mut UdpSocket,
        context: Option<Self::Context>,
    ) -> Result<()> {
        bridge_udp_with_writable(to, self, context).await
    }
}

#[derive(Debug)]
pub struct UdpSession {
    pub address: SocketAddr,
    pub cancel_token: CancellationToken,
}

impl DataBridge<TcpStream> for UdpSocket {
    type Context = UdpSession;
    async fn bridge_to(
        &mut self,
        to: &mut TcpStream,
        context: Option<Self::Context>,
    ) -> Result<()> {
        bridge_udp_with_writable(self, to, context).await
    }
}

impl DataBridge<ServerTlsStream<TcpStream>> for UdpSocket {
    type Context = UdpSession;
    async fn bridge_to(
        &mut self,
        to: &mut ServerTlsStream<TcpStream>,
        context: Option<Self::Context>,
    ) -> Result<()> {
        bridge_udp_with_writable(self, to, context).await
    }
}

async fn bridge_udp_with_writable<T: AsyncWriteExt + Unpin + AsyncReadExt>(
    udp_socket: &mut UdpSocket,
    writable: &mut T,
    context: Option<UdpSession>,
) -> Result<()> {
    let Some(context) = context else {
        return Err(Error::new(ErrorKind::Other, "Context not provided"));
    };

    let mut udp_buffer = BytesMut::with_capacity(2048);
    udp_buffer.resize(2048, 0);

    let mut writable_buffer = BytesMut::with_capacity(2048);
    writable_buffer.resize(2048, 0);

    loop {
        tokio::select! {
            _ = context.cancel_token.cancelled() => {
                break;
            }
            result = udp_socket.recv_from(&mut udp_buffer) => {
                match result {
                    Ok((n, _address)) => {

                        // TODO: See if we can figure how to properly check this. Issue -> 127.0.0.1:8089 != 0.0.0.0:8089
                        // if address != context.address {
                        //     error!("Received data from unexpected address '{}', expected '{}'", address, context.address);
                        //     continue;
                        // }

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
                        context.cancel_token.cancel();
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
                        context.cancel_token.cancel();
                        break;
                    },
                    Ok(n) => {
                        if let Err(e) = udp_socket.send_to(&writable_buffer[..n], context.address).await {
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
                        context.cancel_token.cancel();
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
    type Context = ();

    async fn bridge_to(
        &mut self,
        to: &mut ChannelSocket,
        _context: Option<Self::Context>,
    ) -> Result<()> {
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
