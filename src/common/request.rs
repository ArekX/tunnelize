use std::{
    io::{Error, ErrorKind},
    time::Duration,
};

use tokio::{io::Result, time::timeout};

use super::transport::{read_message, write_message};

pub async fn send_request<Stream, Request, Response>(
    stream: &mut Stream,
    request: &Request,
) -> Result<Response>
where
    Stream: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
    Request: ?Sized + serde::Serialize,
    Response: serde::de::DeserializeOwned,
{
    if let Err(e) = write_message::<Stream, Request>(stream, request).await {
        return Err(Error::new(ErrorKind::Other, e));
    }

    match timeout(
        Duration::from_secs(60),
        read_message::<Stream, Response>(stream),
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
