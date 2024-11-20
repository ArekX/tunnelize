use std::{
    io::{Error, ErrorKind},
    time::Duration,
};

use bytes::BytesMut;
use log::debug;
use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, Result},
    net::TcpStream,
    time::timeout,
};

use super::{
    channel_socket::ChannelSocket,
    data_bridge::DataBridge,
    data_request::DataRequest,
    transport::{read_message, write_message, MessageError},
    udp_client::UdpClient,
};

use tokio_rustls::client::TlsStream as ClientTlsStream;
use tokio_rustls::server::TlsStream as ServerTlsStream;

#[derive(Debug)]
pub enum Connection {
    TcpStream(TcpStream),
    UdpClient(UdpClient),
    TlsStreamServer(ServerTlsStream<TcpStream>),
    TlsStreamClient(ClientTlsStream<TcpStream>),
    ChannelSocket(ChannelSocket),
}

impl From<TcpStream> for Connection {
    fn from(stream: TcpStream) -> Self {
        Self::TcpStream(stream)
    }
}

impl From<UdpClient> for Connection {
    fn from(client: UdpClient) -> Self {
        Self::UdpClient(client)
    }
}

impl From<ServerTlsStream<TcpStream>> for Connection {
    fn from(stream: ServerTlsStream<TcpStream>) -> Self {
        Self::TlsStreamServer(stream)
    }
}

impl From<ClientTlsStream<TcpStream>> for Connection {
    fn from(stream: ClientTlsStream<TcpStream>) -> Self {
        Self::TlsStreamClient(stream)
    }
}

impl From<ChannelSocket> for Connection {
    fn from(socket: ChannelSocket) -> Self {
        Self::ChannelSocket(socket)
    }
}

impl Connection {
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match self {
            Self::TcpStream(stream) => stream.read(buf).await,
            Self::TlsStreamServer(stream) => stream.read(buf).await,
            Self::TlsStreamClient(stream) => stream.read(buf).await,
            Self::UdpClient(client) => client.read(buf).await,
            Self::ChannelSocket(socket) => {
                let data = socket.receive().await?;
                let data_len = data.len();

                buf[..data_len].copy_from_slice(&data);

                Ok(data_len)
            }
        }
    }

    pub async fn read_with_address(
        &mut self,
        buf: &mut [u8],
    ) -> Result<(usize, std::net::SocketAddr)> {
        match self {
            Self::TcpStream(stream) => {
                let Ok(peer_addr) = stream.peer_addr() else {
                    return Err(Error::new(ErrorKind::Other, "Failed to get peer address"));
                };

                let Ok(read_count) = self.read(buf).await else {
                    return Err(Error::new(ErrorKind::Other, "Failed to read from stream"));
                };

                Ok((read_count, peer_addr))
            }
            Self::TlsStreamServer(stream) => {
                let Ok(peer_addr) = stream.get_ref().0.peer_addr() else {
                    return Err(Error::new(ErrorKind::Other, "Failed to get peer address"));
                };

                let Ok(read_count) = self.read(buf).await else {
                    return Err(Error::new(ErrorKind::Other, "Failed to read from stream"));
                };

                Ok((read_count, peer_addr))
            }
            Self::TlsStreamClient(stream) => {
                let Ok(peer_addr) = stream.get_ref().0.peer_addr() else {
                    return Err(Error::new(ErrorKind::Other, "Failed to get peer address"));
                };

                let Ok(read_count) = self.read(buf).await else {
                    return Err(Error::new(ErrorKind::Other, "Failed to read from stream"));
                };

                Ok((read_count, peer_addr))
            }
            Self::UdpClient(_) => {
                return Err(Error::new(
                    ErrorKind::Other,
                    "UDP clients cannot read with address.", // TODO: Do this differently
                ));
            }
            Self::ChannelSocket(_) => {
                return Err(Error::new(
                    ErrorKind::Other,
                    "Channel sockets cannot read with address.",
                ))
            }
        }
    }

    pub async fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        match self {
            Self::TcpStream(stream) => stream.write_all(buf).await,
            Self::TlsStreamServer(stream) => stream.write_all(buf).await,
            Self::TlsStreamClient(stream) => stream.write_all(buf).await,
            Self::UdpClient(client) => client.write(buf).await.map(|_| ()),
            Self::ChannelSocket(socket) => {
                socket.send(buf.to_vec()).await?;
                Ok(())
            }
        }
    }

    pub async fn read_message<Message>(&mut self) -> std::result::Result<Message, MessageError>
    where
        Message: DeserializeOwned,
    {
        match self {
            Self::TcpStream(stream) => read_message(stream).await,
            Self::TlsStreamServer(stream) => read_message(stream).await,
            Self::TlsStreamClient(stream) => read_message(stream).await,
            Self::UdpClient(_) => Err(MessageError::IoError(Error::new(
                ErrorKind::Other,
                "Reading messages from UDP connection is not supported.",
            ))),
            Self::ChannelSocket(socket) => {
                let data = socket.receive().await?;

                Ok(rmp_serde::from_slice(&data)?)
            }
        }
    }

    pub async fn read_string_until(&mut self, until_string: &str) -> String {
        let mut request_buffer = Vec::new();
        let mut buffer = BytesMut::with_capacity(2048);
        buffer.resize(2048, 0);

        loop {
            match self.read(&mut buffer).await {
                Ok(0) => {
                    break;
                }
                Ok(read) => {
                    request_buffer.extend_from_slice(&buffer[..read]);

                    if String::from_utf8_lossy(&request_buffer).contains(until_string) {
                        break;
                    }
                }
                Err(e) => {
                    debug!("Error while reading until block: {:?}", e);
                    break;
                }
            }
        }

        match String::from_utf8(request_buffer) {
            Ok(result) => result,
            Err(e) => {
                debug!("Error while converting buffer to string: {:?}", e);
                String::new()
            }
        }
    }

    pub async fn write_message<Message>(
        &mut self,
        message: &Message,
    ) -> std::result::Result<(), MessageError>
    where
        Message: ?Sized + Serialize,
    {
        match self {
            Self::TcpStream(stream) => write_message(stream, &message).await,
            Self::TlsStreamServer(stream) => write_message(stream, &message).await,
            Self::TlsStreamClient(stream) => write_message(stream, &message).await,
            Self::UdpClient(_) => Err(MessageError::IoError(Error::new(
                ErrorKind::Other,
                "Writing messages to UDP connection is not supported.",
            ))),
            Self::ChannelSocket(socket) => {
                let data = match rmp_serde::to_vec(&message) {
                    Ok(data) => data,
                    Err(e) => {
                        debug!("Error while serializing message: {:?}", e);
                        return Err(MessageError::EncodeError(e));
                    }
                };

                socket.send(data).await?;

                Ok(())
            }
        }
    }

    pub async fn request_message<RequestMessage: DataRequest>(
        &mut self,
        request: RequestMessage,
    ) -> Result<RequestMessage::DataResponse>
    where
        RequestMessage: ?Sized + Serialize + Into<RequestMessage::DataEnum>,
    {
        if let Err(e) = self
            .write_message::<RequestMessage::DataEnum>(&request.into())
            .await
        {
            debug!("Error while sending message: {:?}", e);
            return Err(Error::new(ErrorKind::Other, e));
        }

        match timeout(
            Duration::from_secs(60),
            self.read_message::<RequestMessage::DataResponse>(),
        )
        .await
        {
            Ok(response) => match response {
                Ok(response) => Ok(response),
                Err(e) => {
                    debug!("Error while reading response: {:?}", e);
                    Err(Error::new(ErrorKind::Other, e))
                }
            },
            Err(e) => {
                debug!("Timeout while waiting for response: {:?}", e);
                Err(Error::new(ErrorKind::TimedOut, e))
            }
        }
    }

    pub async fn respond_message<Message>(&mut self, message: &Message)
    where
        Message: ?Sized + Serialize,
    {
        if let Err(e) = self.write_message(message).await {
            debug!("Error while sending message: {:?}", e);
        }
    }

    pub async fn shutdown(&mut self) {
        match self {
            Self::TcpStream(stream) => {
                if let Err(e) = stream.shutdown().await {
                    debug!("Error while closing stream: {:?}", e);
                }
            }
            Self::TlsStreamServer(stream) => {
                if let Err(e) = stream.shutdown().await {
                    debug!("Error while closing stream: {:?}", e);
                }
            }
            Self::TlsStreamClient(stream) => {
                if let Err(e) = stream.shutdown().await {
                    debug!("Error while closing stream: {:?}", e);
                }
            }
            Self::UdpClient(client) => {
                client.shutdown();
            }
            Self::ChannelSocket(socket) => {
                socket.shutdown().await;
            }
        }
    }

    pub async fn close_with_data(&mut self, message: &[u8]) {
        if message.len() > 0 {
            if let Err(e) = self.write_all(message).await {
                debug!("Error while sending message: {:?}", e);
            }
        }

        self.shutdown().await;
    }

    pub fn get_protocol(&self) -> &str {
        match self {
            Self::TcpStream(_) => "tcp",
            Self::TlsStreamServer(_) => "tcp (tls-server)",
            Self::TlsStreamClient(_) => "tcp (tls-client)",
            Self::UdpClient(_) => "udp",
            Self::ChannelSocket(_) => "channel socket",
        }
    }
}

macro_rules! allow_bridges {
    ($self_item: ident, $destination: ident, $context: ident, {
        $($from: ident -> $to: ident),*
    }) => {
        match ($self_item, $destination) {
            $(
                (Self::$from(src), Self::$to(dst)) => src.bridge_to(dst).await,
            )*
            (a, b) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Incompatible Protocol Types for pipe '{}' and '{}'",
                    a.get_protocol(),
                    b.get_protocol()
                ),
            )),
        }
    };
}

impl DataBridge<Connection> for Connection {
    async fn bridge_to(&mut self, to: &mut Connection) -> Result<()> {
        allow_bridges!(self, to, context, {
            TcpStream -> TcpStream,
            TcpStream -> TlsStreamServer,
            TlsStreamServer -> TcpStream,
            TlsStreamServer -> TlsStreamServer,
            TcpStream -> TlsStreamClient,
            UdpClient -> TcpStream,
            TcpStream -> UdpClient,
            UdpClient -> TlsStreamServer,
            TlsStreamServer -> UdpClient,
            UdpClient -> TlsStreamClient,
            TcpStream -> ChannelSocket,
            TlsStreamServer -> ChannelSocket
        })
    }
}
