use std::{
    io::{Error, ErrorKind},
    ops::ControlFlow,
    time::Duration,
};

use log::debug;
use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, Result},
    net::TcpStream,
    time::timeout,
};
use tokio_rustls::client::TlsStream;

use super::transport::{read_message, write_message, MessageError};

pub enum ConnectionStream {
    TcpStream(TcpStream),
    TlsStream(TlsStream<TcpStream>),
}

impl ConnectionStream {
    pub fn from_tcp_stream(stream: TcpStream) -> Self {
        Self::TcpStream(stream)
    }

    pub fn from_tls_stream(stream: TlsStream<TcpStream>) -> Self {
        Self::TlsStream(stream)
    }

    pub async fn wait_for_messages(&mut self) -> Result<ControlFlow<()>> {
        let mut buf = [0; 1];

        let inner_stream = match self {
            Self::TcpStream(stream) => stream,
            Self::TlsStream(stream) => &mut stream.get_mut().0,
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
            Self::TlsStream(stream) => stream.read(buf).await,
        }
    }

    pub async fn write_all(&mut self, buf: &[u8]) -> Result<()> {
        match self {
            Self::TcpStream(stream) => stream.write_all(buf).await,
            Self::TlsStream(stream) => stream.write_all(buf).await,
        }
    }

    pub async fn read_message<Message>(&mut self) -> std::result::Result<Message, MessageError>
    where
        Message: DeserializeOwned,
    {
        match self {
            Self::TcpStream(stream) => read_message(stream).await,
            Self::TlsStream(stream) => read_message(stream).await,
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
            Self::TlsStream(stream) => write_message(stream, &message).await,
        }
    }

    pub async fn request<Request, Response>(
        &mut self,
        request: &Request,
    ) -> std::result::Result<Response, Error>
    where
        Request: ?Sized + Serialize,
        Response: DeserializeOwned,
    {
        if let Err(e) = self.write_message::<Request>(request).await {
            return Err(Error::new(ErrorKind::Other, e));
        }

        match timeout(Duration::from_secs(60), self.read_message::<Response>()).await {
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
}
