use std::{
    io::{Error, ErrorKind},
    net::SocketAddr,
};

use log::{debug, error};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, Result},
    net::{TcpStream, UdpSocket},
};
use tokio_rustls::client::TlsStream;
use tokio_util::sync::CancellationToken;

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

impl DataBridge<TlsStream<TcpStream>> for TcpStream {
    type Context = ();
    async fn bridge_to(
        &mut self,
        to: &mut TlsStream<TcpStream>,
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

impl DataBridge<TcpStream> for TlsStream<TcpStream> {
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

impl DataBridge<TlsStream<TcpStream>> for TlsStream<TcpStream> {
    type Context = ();
    async fn bridge_to(
        &mut self,
        to: &mut TlsStream<TcpStream>,
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

impl DataBridge<UdpSocket> for TlsStream<TcpStream> {
    type Context = UdpSession;
    async fn bridge_to(
        &mut self,
        to: &mut UdpSocket,
        context: Option<Self::Context>,
    ) -> Result<()> {
        bridge_udp_with_writable(to, self, context).await
    }
}

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

impl DataBridge<TlsStream<TcpStream>> for UdpSocket {
    type Context = UdpSession;
    async fn bridge_to(
        &mut self,
        to: &mut TlsStream<TcpStream>,
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

    let mut udp_buffer = [0u8; 65537];
    let mut tcp_buffer = [0u8; 65537];

    loop {
        tokio::select! {
            _ = context.cancel_token.cancelled() => {
                break;
            }
            result = udp_socket.recv_from(&mut udp_buffer) => {
                match result {
                    Ok((n, address)) => {

                        if address != context.address {
                            error!("Received data from unexpected address: {}", address);
                            continue;
                        }

                        if let Err(e) = writable.write_all(&udp_buffer[..n]).await {
                            error!("Failed to send data to TCP stream: {}", e);
                        }
                    }
                    Err(e)
                        if e.kind() == std::io::ErrorKind::UnexpectedEof
                            || e.kind() == std::io::ErrorKind::ConnectionAborted
                            || e.kind() == std::io::ErrorKind::ConnectionReset
                            || e.kind() == std::io::ErrorKind::BrokenPipe =>
                    {
                        debug!("TCP <-> UDP connection ended: {:?}", e);
                        context.cancel_token.cancel();
                        break;
                    }
                    Err(e) => {
                        error!("Failed to receive data from UDP socket: {}", e);
                    }
                }
            }
            result = writable.read(&mut tcp_buffer) => {
                match result {
                    Ok(0) => {
                        context.cancel_token.cancel();
                        break;
                    },
                    Ok(n) => {
                        if let Err(e) = udp_socket.send_to(&tcp_buffer[..n], context.address).await {
                            error!("Failed to send data to UDP socket: {}", e);
                        }
                    }
                    Err(e)
                        if e.kind() == std::io::ErrorKind::UnexpectedEof
                            || e.kind() == std::io::ErrorKind::ConnectionAborted
                            || e.kind() == std::io::ErrorKind::ConnectionReset
                            || e.kind() == std::io::ErrorKind::BrokenPipe =>
                    {
                        debug!("TCP <-> UDP connection ended: {:?}", e);
                        context.cancel_token.cancel();
                        break;
                    }
                    Err(e) => {
                        error!("Failed to receive data from TCP stream: {}", e);
                    }
                }
            }
        }
    }

    Ok(())
}
