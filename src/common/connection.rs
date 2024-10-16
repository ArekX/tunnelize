use std::{
    io::{Error, ErrorKind},
    ops::ControlFlow,
    time::Duration,
};

use log::debug;
use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt, Result},
    net::TcpStream,
    time::timeout,
};
use tokio_rustls::client::TlsStream;

use super::transport::{read_message, write_message, MessageError};

#[derive(Debug)]
pub enum ConnectionStream {
    TcpStream(TcpStream),
    TlsTcpStream(TlsStream<TcpStream>),
}

impl From<TcpStream> for ConnectionStream {
    fn from(stream: TcpStream) -> Self {
        Self::TcpStream(stream)
    }
}

impl From<TlsStream<TcpStream>> for ConnectionStream {
    fn from(stream: TlsStream<TcpStream>) -> Self {
        Self::TlsTcpStream(stream)
    }
}

impl ConnectionStream {
    pub async fn wait_for_messages(&mut self) -> Result<ControlFlow<()>> {
        let mut buf = [0; 1];

        let inner_stream = match self {
            Self::TcpStream(stream) => stream,
            Self::TlsTcpStream(stream) => stream.get_mut().0,
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
        }
    }

    pub async fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        match self {
            Self::TcpStream(stream) => stream.write_all(buf).await,
            Self::TlsTcpStream(stream) => stream.write_all(buf).await,
        }
    }

    pub async fn read_message<Message>(&mut self) -> std::result::Result<Message, MessageError>
    where
        Message: DeserializeOwned,
    {
        match self {
            Self::TcpStream(stream) => read_message(stream).await,
            Self::TlsTcpStream(stream) => read_message(stream).await,
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
        }
    }

    pub async fn request_message<RequestMessage, ResponseMessage>(
        &mut self,
        request: &RequestMessage,
    ) -> std::result::Result<ResponseMessage, Error>
    where
        RequestMessage: ?Sized + Serialize,
        ResponseMessage: DeserializeOwned,
    {
        if let Err(e) = self.write_message::<RequestMessage>(request).await {
            return Err(Error::new(ErrorKind::Other, e));
        }

        match timeout(
            Duration::from_secs(60),
            self.read_message::<ResponseMessage>(),
        )
        .await
        {
            Ok(response) => match response {
                Ok(response) => Ok(response),
                Err(e) => Err(Error::new(ErrorKind::Other, e)),
            },
            Err(e) => Err(Error::new(ErrorKind::Other, e)),
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
        }
    }

    pub async fn write_and_shutdown(&mut self, message: &[u8]) {
        if let Err(e) = self.write_all(message).await {
            debug!("Error while sending message: {:?}", e);
        }

        self.shutdown().await;
    }

    pub async fn link_session_with(&mut self, other: &mut Self) -> Result<()> {
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
        }
    }
}
