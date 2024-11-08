use std::{
    io::{Error, ErrorKind},
    net::SocketAddr,
};

use tokio::net::{TcpListener, TcpStream};

use tokio::io::Result;

use super::{
    configuration::ServerEncryption, connection::Connection, encryption::ServerTlsEncryption,
};

pub struct TcpServer {
    encryption: Option<ServerTlsEncryption>,
    listener: TcpListener,
}

impl TcpServer {
    pub async fn new(address: String, port: u16, encryption: ServerEncryption) -> Result<Self> {
        Ok(TcpServer {
            encryption: match encryption {
                ServerEncryption::None => None,
                ServerEncryption::Tls {
                    cert_path: cert,
                    key_path: key,
                } => Some(ServerTlsEncryption::new(&cert, &key).await),
            },
            listener: TcpListener::bind(format!("{}:{}", address, port)).await?,
        })
    }

    pub async fn listen_for_connection(
        &self,
    ) -> core::result::Result<(Connection, SocketAddr), (Error, Option<Connection>)> {
        let (stream, addr) = self.listener.accept().await.map_err(|e| (e, None))?;

        if self.encryption.is_some() && !is_tls_stream(&stream).await.map_err(|e| (e, None))? {
            return Err((
                Error::new(ErrorKind::InvalidData, "Not TLS"),
                Some(Connection::from(stream)),
            ));
        }

        match self.encryption {
            Some(ref tls) => Ok((tls.accept(stream).await.map_err(|e| (e, None))?, addr)),
            None => Ok((Connection::from(stream), addr)),
        }
    }
}

async fn is_tls_stream(stream: &TcpStream) -> Result<bool> {
    let mut peek_buf = [0u8; 5]; // TLS record header is 5 bytes

    // Peek at the first bytes without consuming them
    match stream.peek(&mut peek_buf).await? {
        n if n >= 5 => {
            // TLS record format:
            // Byte 0: Content Type (0x16 for Handshake)
            // Bytes 1-2: Version (0x0301 for TLS 1.0, 0x0302 for TLS 1.1, 0x0303 for TLS 1.2/1.3)
            // Bytes 3-4: Length

            let content_type = peek_buf[0];
            let version_major = peek_buf[1];
            let version_minor = peek_buf[2];

            // Check if it matches TLS handshake patterns
            Ok(content_type == 0x16 && // Handshake
                version_major == 0x03 && // TLS version
                (version_minor >= 0x01 && version_minor <= 0x04)) // TLS 1.0 through 1.3
        }
        _ => Ok(false),
    }
}
