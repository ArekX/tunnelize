use std::{
    io::{Error, ErrorKind},
    ops::ControlFlow,
    time::Duration,
};

use log::debug;
use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt, Result},
    net::{TcpStream, UdpSocket},
    time::timeout,
};
use tokio_rustls::client::TlsStream;

use super::{
    data_request::DataRequest,
    transport::{read_message, write_message, MessageError},
};

#[derive(Debug)]
pub enum ConnectionStream {
    TcpStream(TcpStream),
    UdpSocket(UdpSocket),
    TlsTcpStream(TlsStream<TcpStream>),
    // TODO: Add TlsUdpStream
}

impl From<TcpStream> for ConnectionStream {
    fn from(stream: TcpStream) -> Self {
        Self::TcpStream(stream)
    }
}

impl From<UdpSocket> for ConnectionStream {
    fn from(socket: UdpSocket) -> Self {
        Self::UdpSocket(socket)
    }
}

impl From<TlsStream<TcpStream>> for ConnectionStream {
    fn from(stream: TlsStream<TcpStream>) -> Self {
        Self::TlsTcpStream(stream)
    }
}

impl ConnectionStream {
    pub async fn wait_for_data(&mut self) -> Result<ControlFlow<()>> {
        let mut buf = [0; 1];

        let inner_stream = match self {
            Self::TcpStream(stream) => stream,
            Self::TlsTcpStream(stream) => stream.get_mut().0,
            Self::UdpSocket(_) => {
                // TODO: Implement this for UdpSocket
                return Ok(ControlFlow::Continue(()));
            }
        };

        match inner_stream.peek(&mut buf).await {
            Ok(0) => Ok(ControlFlow::Break(())),
            Ok(_) => Ok(ControlFlow::Continue(())),
            Err(e) => Err(e),
        }
    }

    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match self {
            Self::TcpStream(stream) => stream.read(buf).await,
            Self::TlsTcpStream(stream) => stream.read(buf).await,
            Self::UdpSocket(socket) => socket.recv(buf).await,
        }
    }

    pub async fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        match self {
            Self::TcpStream(stream) => stream.write_all(buf).await,
            Self::TlsTcpStream(stream) => stream.write_all(buf).await,
            Self::UdpSocket(socket) => socket.send(buf).await.map(|_| ()),
        }
    }

    pub async fn read_message<Message>(&mut self) -> std::result::Result<Message, MessageError>
    where
        Message: DeserializeOwned,
    {
        match self {
            Self::TcpStream(stream) => read_message(stream).await,
            Self::TlsTcpStream(stream) => read_message(stream).await,
            Self::UdpSocket(_) => {
                todo!("Implement read_message for UdpSocket");
            }
        }
    }

    pub async fn read_string_until(&mut self, until_string: &str) -> String {
        let mut request_buffer = Vec::new();

        loop {
            debug!("Waiting tcp stream to be readable...");

            if let Err(e) = self.wait_for_data().await {
                debug!(
                    "Error while waiting for client stream to be readable: {:?}",
                    e
                );
                break;
            }

            let mut buffer = [0; 100024];

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
            Self::TlsTcpStream(stream) => write_message(stream, &message).await,
            Self::UdpSocket(_) => {
                todo!("Implement write_message for UdpSocket");
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
            Self::TlsTcpStream(stream) => {
                if let Err(e) = stream.shutdown().await {
                    debug!("Error while closing stream: {:?}", e);
                }
            }
            Self::UdpSocket(_) => {
                // No close for UdpSocket
            }
        }
    }

    pub async fn close_with_data(&mut self, message: &[u8]) {
        if let Err(e) = self.write_all(message).await {
            debug!("Error while sending message: {:?}", e);
        }

        self.shutdown().await;
    }

    pub async fn pipe_to(&mut self, other: &mut Self) -> Result<()> {
        // TODO: TcpStream to TlsStream will probably need to be handled differently
        match (self, other) {
            (Self::TcpStream(src), Self::TcpStream(dst)) => {
                match io::copy_bidirectional(src, dst).await {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e),
                }
            }
            (Self::TlsTcpStream(src), Self::TlsTcpStream(dst)) => {
                match io::copy_bidirectional(&mut src.get_mut().0, &mut dst.get_mut().0).await {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e),
                }
            }
            (a, b) => Err(Error::new(
                ErrorKind::Other,
                format!(
                    "Incompatible Protocol Types '{}' and '{}'",
                    a.get_protocol(),
                    b.get_protocol()
                ),
            )),
        }
    }

    pub fn get_protocol(&self) -> &str {
        match self {
            Self::TcpStream(_) => "tcp",
            Self::TlsTcpStream(_) => "tcp (tls)",
            Self::UdpSocket(_) => "udp",
        }
    }
}
