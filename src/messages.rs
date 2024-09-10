use std::fmt;

use bincode;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;

#[derive(Serialize, Deserialize)]
pub enum Message {
    Connect,
    LinkRequest { id: u32 },
    LinkAccept { id: u32 },
}

#[derive(Debug)]
pub enum MessageError {
    SerializationError(bincode::Error),
    IoError(std::io::Error),
    InvalidLength(u32),
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
        }
    }
}

impl std::error::Error for MessageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MessageError::SerializationError(e) => Some(e),
            MessageError::IoError(e) => Some(e),
            MessageError::InvalidLength(_) => None,
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

fn serialize_message(message: &Message) -> Result<Bytes, MessageError> {
    let encoded: Vec<u8> = bincode::serialize(message)?;
    let mut bytes = BytesMut::with_capacity(encoded.len());
    bytes.put_slice(&encoded);
    Ok(bytes.freeze())
}

fn deserialize_message(bytes: &Bytes) -> Result<Message, MessageError> {
    let message: Message = bincode::deserialize(bytes)?;
    Ok(message)
}

async fn read_exact<T: AsyncReadExt + Unpin>(
    stream: &mut T,
    length: usize,
) -> Result<Bytes, MessageError> {
    let mut buffer = BytesMut::with_capacity(length);
    buffer.resize(length, 0);
    stream.read_exact(&mut buffer).await?;
    Ok(buffer.freeze())
}

pub async fn read_message<T: AsyncReadExt + Unpin>(
    stream: &mut T,
) -> Result<Message, MessageError> {
    let mut length_bytes = read_exact(stream, 4).await?;
    let length = length_bytes.get_u32();

    if (length) > MAX_MESSAGE_LENGTH {
        return Err(MessageError::InvalidLength(length));
    }

    // Read the message data
    let message_bytes = read_exact(stream, length as usize).await?;
    let message = deserialize_message(&message_bytes)?;

    Ok(message)
}

pub async fn write_message<T: tokio::io::AsyncWriteExt + Unpin>(
    stream: &mut T,
    message: &Message,
) -> Result<(), MessageError> {
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
