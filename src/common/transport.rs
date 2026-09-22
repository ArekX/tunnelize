use std::{fmt, time::Duration};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use log::debug;
use serde::de::DeserializeOwned;
use tokio::{
    io::{AsyncRead, AsyncReadExt},
    time::Instant,
};

#[derive(Debug)]
pub enum MessageError {
    EncodeError(rmp_serde::encode::Error),
    DecodeError(rmp_serde::decode::Error),
    IoError(std::io::Error),
    InvalidLength(u32),
    ConnectionClosed,
}

pub const MAX_MESSAGE_LENGTH: u32 = 50 * 1024 * 1024; // 50MB
const MESSAGE_HEADER_LENGTH: usize = 4;
const MAX_RECEIVE_TIMEOUT: Duration = Duration::from_secs(30);

impl fmt::Display for MessageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageError::EncodeError(e) => write!(f, "Encoding error: {e}"),
            MessageError::DecodeError(e) => write!(f, "Decoding error: {e}"),
            MessageError::IoError(e) => write!(f, "IO error: {e}"),
            MessageError::InvalidLength(length) => {
                write!(
                    f,
                    "Message longer than {} bytes. Length: {length} bytes.",
                    MAX_MESSAGE_LENGTH
                )
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

/// Holds the bytes of a partially received frame between calls to [`read_message`].
///
/// `read_message` is typically awaited inside `tokio::select!`. When another branch
/// wins, the read future is dropped. Any bytes it had already pulled off the socket
/// must survive that drop, otherwise the stream desynchronises and the next read
/// interprets the middle of a message as a length header. Keeping the partial frame
/// here (and only ever reading with the cancel-safe `read_buf`) makes the read
/// resumable.
#[derive(Debug, Default)]
pub struct MessageReadState {
    buffer: BytesMut,
    frame_started_at: Option<Instant>,
}

impl MessageReadState {
    pub fn new() -> Self {
        Self::default()
    }

    fn reset(&mut self) {
        self.buffer.clear();
        self.frame_started_at = None;
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty() && self.frame_started_at.is_none()
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
    Ok(rmp_serde::from_slice(&bytes)?)
}

fn map_read_error(e: std::io::Error) -> MessageError {
    match e.kind() {
        std::io::ErrorKind::UnexpectedEof
        | std::io::ErrorKind::ConnectionAborted
        | std::io::ErrorKind::ConnectionReset
        | std::io::ErrorKind::BrokenPipe => {
            debug!("Cannot read message, connection closed: {:?}", e);
            MessageError::ConnectionClosed
        }
        _ => {
            debug!("Error reading message. {:?}", e);
            MessageError::IoError(e)
        }
    }
}

/// Reads at most `max_length` bytes into `buffer`. Cancel-safe: if the returned
/// future is dropped before completion, no bytes were consumed from the stream.
async fn read_chunk<T: AsyncRead + Unpin>(
    stream: &mut T,
    buffer: &mut BytesMut,
    max_length: usize,
) -> Result<usize, MessageError> {
    buffer.reserve(max_length);

    match stream.read_buf(&mut buffer.limit(max_length)).await {
        Ok(0) => Err(MessageError::ConnectionClosed),
        Ok(read) => Ok(read),
        Err(e) => Err(map_read_error(e)),
    }
}

/// Grows the buffered frame until it holds at least `target_length` bytes without
/// ever reading past that point, so that bytes belonging to whatever follows the
/// message (e.g. a bridged data stream) are left untouched on the socket.
async fn fill_buffer<T: AsyncRead + Unpin>(
    stream: &mut T,
    state: &mut MessageReadState,
    target_length: usize,
) -> Result<(), MessageError> {
    while state.buffer.len() < target_length {
        let remaining = target_length - state.buffer.len();

        match state.frame_started_at {
            // Waiting for the start of a new message can take arbitrarily long.
            None => {
                read_chunk(stream, &mut state.buffer, remaining).await?;
                state.frame_started_at = Some(Instant::now());
            }
            // Once a frame has started, the rest of it must arrive within the timeout.
            Some(started_at) => {
                let deadline = started_at + MAX_RECEIVE_TIMEOUT;
                let read = read_chunk(stream, &mut state.buffer, remaining);

                match tokio::time::timeout_at(deadline, read).await {
                    Ok(result) => {
                        result?;
                    }
                    Err(_) => {
                        return Err(MessageError::IoError(std::io::Error::new(
                            std::io::ErrorKind::TimedOut,
                            "Timed out while reading message",
                        )));
                    }
                }
            }
        }
    }

    Ok(())
}

async fn read_frame<T: AsyncRead + Unpin>(
    stream: &mut T,
    state: &mut MessageReadState,
) -> Result<Bytes, MessageError> {
    fill_buffer(stream, state, MESSAGE_HEADER_LENGTH).await?;

    let length = u32::from_be_bytes([
        state.buffer[0],
        state.buffer[1],
        state.buffer[2],
        state.buffer[3],
    ]);

    if length > MAX_MESSAGE_LENGTH {
        return Err(MessageError::InvalidLength(length));
    }

    let frame_length = MESSAGE_HEADER_LENGTH + length as usize;

    fill_buffer(stream, state, frame_length).await?;

    let mut frame = state.buffer.split_to(frame_length);
    frame.advance(MESSAGE_HEADER_LENGTH);
    state.frame_started_at = None;

    Ok(frame.freeze())
}

/// Reads one length-prefixed message from `stream`.
///
/// This function is cancel-safe as long as the same `state` is passed on the next
/// call for the same stream: a partially received frame is kept in `state` and the
/// read resumes where it left off. After an error the partial state is discarded,
/// since the stream framing can no longer be trusted.
pub async fn read_message<T: AsyncRead + Unpin, M>(
    stream: &mut T,
    state: &mut MessageReadState,
) -> Result<M, MessageError>
where
    M: DeserializeOwned,
{
    match read_frame(stream, state).await {
        Ok(frame) => deserialize_message(frame),
        Err(e) => {
            state.reset();
            Err(e)
        }
    }
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

    let mut length_bytes = BytesMut::with_capacity(MESSAGE_HEADER_LENGTH);
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

    fn frame(message: &TestMessage) -> Vec<u8> {
        let serialized = serialize_message(message).unwrap();
        let mut framed = Vec::new();
        framed.extend_from_slice(&(serialized.len() as u32).to_be_bytes());
        framed.extend_from_slice(&serialized);
        framed
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

        let mut stream = Cursor::new(frame(&message));
        let mut state = MessageReadState::new();

        let deserialized: TestMessage = read_message(&mut stream, &mut state).await.unwrap();
        assert_eq!(message, deserialized);
        assert!(state.is_empty());
    }

    #[tokio::test]
    async fn test_read_message_does_not_consume_bytes_after_frame() {
        let message = TestMessage {
            content: "Hello, world!".to_string(),
        };

        let mut data = frame(&message);
        let trailing = b"raw bytes that belong to the next stage".to_vec();
        data.extend_from_slice(&trailing);

        let mut stream = Cursor::new(data);
        let mut state = MessageReadState::new();

        let deserialized: TestMessage = read_message(&mut stream, &mut state).await.unwrap();
        assert_eq!(message, deserialized);
        assert!(state.is_empty());

        let mut rest = Vec::new();
        stream.read_to_end(&mut rest).await.unwrap();
        assert_eq!(rest, trailing);
    }

    #[tokio::test]
    async fn test_read_message_resumes_after_cancellation() {
        let message = TestMessage {
            content: "A message that arrives in several pieces".to_string(),
        };
        let framed = frame(&message);

        let (mut writer, mut reader) = tokio::io::duplex(1024);
        let mut state = MessageReadState::new();

        // Deliver the frame in pieces. Between pieces, poll the read and then drop it
        // (like a losing `tokio::select!` branch) to prove the partial frame survives.
        let splits = [0usize, 1, 3, 4, 9, framed.len()];
        for pair in splits.windows(2) {
            writer.write_all(&framed[pair[0]..pair[1]]).await.unwrap();

            if pair[1] < framed.len() {
                let cancelled = tokio::time::timeout(
                    Duration::from_millis(20),
                    read_message::<_, TestMessage>(&mut reader, &mut state),
                )
                .await;
                assert!(cancelled.is_err(), "read must not complete on a partial frame");
            }
        }

        let deserialized: TestMessage = read_message(&mut reader, &mut state).await.unwrap();
        assert_eq!(message, deserialized);
        assert!(state.is_empty());
    }

    #[tokio::test]
    async fn test_read_message_rejects_too_long_message() {
        let mut data = Vec::new();
        data.extend_from_slice(&(MAX_MESSAGE_LENGTH + 1).to_be_bytes());

        let mut stream = Cursor::new(data);
        let mut state = MessageReadState::new();

        let result = read_message::<_, TestMessage>(&mut stream, &mut state).await;
        assert!(matches!(result, Err(MessageError::InvalidLength(_))));
        assert!(state.is_empty());
    }

    #[tokio::test]
    async fn test_read_message_connection_closed_mid_frame() {
        let message = TestMessage {
            content: "Hello, world!".to_string(),
        };
        let framed = frame(&message);

        let mut stream = Cursor::new(framed[..framed.len() - 2].to_vec());
        let mut state = MessageReadState::new();

        let result = read_message::<_, TestMessage>(&mut stream, &mut state).await;
        assert!(matches!(result, Err(MessageError::ConnectionClosed)));
        assert!(state.is_empty());
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

    #[tokio::test]
    async fn test_write_then_read_roundtrip() {
        let message = TestMessage {
            content: "Round trip".to_string(),
        };

        let mut stream = Cursor::new(Vec::new());
        write_message(&mut stream, &message).await.unwrap();
        write_message(&mut stream, &message).await.unwrap();
        stream.set_position(0);

        let mut state = MessageReadState::new();
        let first: TestMessage = read_message(&mut stream, &mut state).await.unwrap();
        let second: TestMessage = read_message(&mut stream, &mut state).await.unwrap();
        assert_eq!(first, message);
        assert_eq!(second, message);
    }
}
