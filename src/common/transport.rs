use std::fmt;

use bytes::{Buf, BufMut, Bytes, BytesMut};
use log::debug;
use serde::de::DeserializeOwned;
use tokio::io::AsyncReadExt;

#[derive(Debug)]
pub enum MessageError {
    EncodeError(rmp_serde::encode::Error),
    DecodeError(rmp_serde::decode::Error),
    IoError(std::io::Error),
    InvalidLength(u32),
    ConnectionClosed,
}

const MAX_MESSAGE_LENGTH: u32 = 10000000; // 10MB

impl fmt::Display for MessageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageError::EncodeError(e) => write!(f, "Encoding error: {}", e),
            MessageError::DecodeError(e) => write!(f, "Decoding error: {}", e),
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
            MessageError::EncodeError(e) => Some(e),
            MessageError::DecodeError(e) => Some(e),
            MessageError::IoError(e) => Some(e),
            MessageError::InvalidLength(_) => None,
            MessageError::ConnectionClosed => None,
        }
    }
}

impl From<rmp_serde::encode::Error> for MessageError {
    fn from(err: rmp_serde::encode::Error) -> MessageError {
        MessageError::EncodeError(err)
    }
}

impl From<rmp_serde::decode::Error> for MessageError {
    fn from(err: rmp_serde::decode::Error) -> MessageError {
        MessageError::DecodeError(err)
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
    let encoded: Vec<u8> = rmp_serde::to_vec(message)?;
    let mut bytes = BytesMut::with_capacity(encoded.len());
    bytes.put_slice(&encoded);
    Ok(bytes.freeze())
}

fn deserialize_message<T>(bytes: Bytes) -> Result<T, MessageError>
where
    T: serde::de::DeserializeOwned,
{
    Ok(rmp_serde::from_slice(&bytes.to_vec())?)
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
        Ok(_) => {
            return Ok(buffer.freeze());
        }
        Err(e)
            if e.kind() == std::io::ErrorKind::UnexpectedEof
                || e.kind() == std::io::ErrorKind::ConnectionAborted
                || e.kind() == std::io::ErrorKind::ConnectionReset
                || e.kind() == std::io::ErrorKind::BrokenPipe =>
        {
            debug!("Cannot read message, connection closed: {:?}", e);
            return Err(MessageError::ConnectionClosed);
        }
        Err(e) => {
            debug!("Error reading message. {:?}", e);
            return Err(MessageError::IoError(e));
        }
    }
}

pub async fn read_message<T: AsyncReadExt + Unpin, M>(stream: &mut T) -> Result<M, MessageError>
where
    M: DeserializeOwned,
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
    let length: u32 = message_bytes.len() as u32;

    if length > MAX_MESSAGE_LENGTH {
        return Err(MessageError::InvalidLength(length));
    }

    let mut length_bytes = BytesMut::with_capacity(4);
    length_bytes.put_u32(length);

    stream.write_all(&length_bytes).await?;
    stream.write_all(&message_bytes).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;
    use bytes::BytesMut;
    use serde::{Deserialize, Serialize};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct TestMessage {
        content: String,
    }

    #[tokio::test]
    async fn test_serialize_message() {
        let message = TestMessage {
            content: "Hello, world!".to_string(),
        };
        let serialized = serialize_message(&message).unwrap();
        assert!(!serialized.is_empty());
    }

    #[tokio::test]
    async fn test_deserialize_message() {
        let message = TestMessage {
            content: "Hello, world!".to_string(),
        };
        let serialized = serialize_message(&message).unwrap();
        let deserialized: TestMessage = deserialize_message(serialized).unwrap();
        assert_eq!(message, deserialized);
    }

    #[tokio::test]
    async fn test_read_message() {
        let message = TestMessage {
            content: "Hello, world!".to_string(),
        };
        let serialized = serialize_message(&message).unwrap();
        let length = serialized.len() as u32;

        let mut stream = Cursor::new(Vec::new());
        stream.write_u32(length).await.unwrap();
        stream.write_all(&serialized).await.unwrap();
        stream.set_position(0);

        let deserialized: TestMessage = read_message(&mut stream).await.unwrap();
        assert_eq!(message, deserialized);
    }

    #[tokio::test]
    async fn test_write_message() {
        let message = TestMessage {
            content: "Hello, world!".to_string(),
        };

        let mut stream = Cursor::new(Vec::new());
        write_message(&mut stream, &message).await.unwrap();
        stream.set_position(0);

        let length = stream.read_u32().await.unwrap();
        assert_eq!(length, serialize_message(&message).unwrap().len() as u32);

        let mut buffer = BytesMut::with_capacity(length as usize);
        buffer.resize(length as usize, 0);
        stream.read_exact(&mut buffer).await.unwrap();

        let deserialized: TestMessage = deserialize_message(buffer.freeze()).unwrap();
        assert_eq!(message, deserialized);
    }
}
