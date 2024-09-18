use std::fmt;

use bincode;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;

#[derive(Serialize, Deserialize)]
pub struct TunnelClientRequest {
    pub name: Option<String>,
    pub forward_address: String,
}

#[derive(Serialize, Deserialize)]
pub enum TunnelMessage {
    Connect {
        client_requests: Vec<TunnelClientRequest>,
    },
    Disconnect {
        tunnel_id: u32,
    },
    LinkDeny {
        tunnel_id: u32,
        id: u32,
        reason: String,
    },
    LinkAccept {
        tunnel_id: u32,
        id: u32,
    },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ResolvedLink {
    pub forward_address: String,
    pub client_address: String,
    pub link_id: u32,
}

#[derive(Serialize, Deserialize)]
pub enum ServerMessage {
    ConnectAccept {
        tunnel_id: u32,
        resolved_links: Vec<ResolvedLink>,
    },
    LinkRequest {
        id: u32,
        link_id: u32,
    },
}

#[derive(Debug)]
pub enum MessageError {
    SerializationError(bincode::Error),
    IoError(std::io::Error),
    InvalidLength(u32),
    ConnectionClosed,
}

const MAX_MESSAGE_LENGTH: u32 = 10000000; // 10MB

impl fmt::Display for MessageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            MessageError::IoError(e) => write!(f, "IO error: {}", e),
            MessageError::InvalidLength(length) => {
                write!(f, "Message longer than 10MB. Length: {} bytes.", length)
            }
            MessageError::ConnectionClosed => write!(f, "Connection closed."),
        }
    }
}

impl std::error::Error for MessageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MessageError::SerializationError(e) => Some(e),
            MessageError::IoError(e) => Some(e),
            MessageError::InvalidLength(_) => None,
            MessageError::ConnectionClosed => None,
        }
    }
}

impl From<bincode::Error> for MessageError {
    fn from(err: bincode::Error) -> MessageError {
        MessageError::SerializationError(err)
    }
}

impl From<std::io::Error> for MessageError {
    fn from(err: std::io::Error) -> MessageError {
        MessageError::IoError(err)
    }
}

fn serialize_message<T>(message: &T) -> Result<Bytes, MessageError>
where
    T: ?Sized + serde::Serialize,
{
    let encoded: Vec<u8> = bincode::serialize(message)?;
    let mut bytes = BytesMut::with_capacity(encoded.len());
    bytes.put_slice(&encoded);
    Ok(bytes.freeze())
}

fn deserialize_message<T>(bytes: Bytes) -> Result<T, MessageError>
where
    T: serde::de::DeserializeOwned,
{
    Ok(bincode::deserialize(&bytes)?)
}

async fn read_exact<T: AsyncReadExt + Unpin>(
    stream: &mut T,
    length: usize,
) -> Result<Bytes, MessageError> {
    let mut buffer = BytesMut::with_capacity(length);
    buffer.resize(length, 0);
    match stream.read_exact(&mut buffer).await {
        Ok(0) => {
            return Err(MessageError::ConnectionClosed);
        }
        Ok(_) => return Ok(buffer.freeze()),
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
            return Err(MessageError::ConnectionClosed);
        }
        Err(e) => {
            return Err(MessageError::IoError(e));
        }
    }
}

pub async fn read_message<T: AsyncReadExt + Unpin, M>(stream: &mut T) -> Result<M, MessageError>
where
    M: serde::de::DeserializeOwned,
{
    let mut length_bytes = read_exact(stream, 4).await?;
    let length = length_bytes.get_u32();

    if (length) > MAX_MESSAGE_LENGTH {
        return Err(MessageError::InvalidLength(length));
    }

    let message_bytes = read_exact(stream, length as usize).await?;

    deserialize_message(message_bytes)
}

pub async fn write_message<T: tokio::io::AsyncWriteExt + Unpin, M>(
    stream: &mut T,
    message: &M,
) -> Result<(), MessageError>
where
    M: ?Sized + serde::Serialize,
{
    let message_bytes = serialize_message(message)?;
    let length = message_bytes.len() as u32;

    if length > MAX_MESSAGE_LENGTH {
        return Err(MessageError::InvalidLength(length));
    }

    let mut length_bytes = BytesMut::with_capacity(4);
    length_bytes.put_u32(length);

    stream.write_all(&length_bytes).await?;
    stream.write_all(&message_bytes).await?;

    Ok(())
}
