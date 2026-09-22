use std::{
    io::{Error, ErrorKind},
    net::SocketAddr,
    sync::Arc,
    time::Duration,
};

use tokio::net::{TcpListener, TcpStream};

use tokio::io::Result;

use super::{
    configuration::ServerEncryption, connection::Connection, encryption::ServerTlsEncryption,
    listener::bind_tcp_listener, protocol_socket::unmap_socket_addr,
};

const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);

pub struct TcpServer {
    encryption: Arc<Option<ServerTlsEncryption>>,
    listener: TcpListener,
}

pub struct PendingConnection {
    stream: TcpStream,
    encryption: Arc<Option<ServerTlsEncryption>>,
}

impl TcpServer {
    pub async fn new(address: String, port: u16, encryption: ServerEncryption) -> Result<Self> {
        Ok(TcpServer {
            encryption: Arc::new(match encryption {
                ServerEncryption::None => None,
                ServerEncryption::Tls {
                    cert_path: cert,
                    key_path: key,
                } => Some(ServerTlsEncryption::new(&cert, &key).await),
            }),
            listener: bind_tcp_listener(&address, port).await?,
        })
    }

    pub async fn listen_for_connection(&self) -> Result<(PendingConnection, SocketAddr)> {
        let (stream, addr) = self.listener.accept().await?;

        Ok((
            PendingConnection {
                stream,
                encryption: self.encryption.clone(),
            },
            // On a dual-stack listener IPv4 peers arrive as `::ffff:a.b.c.d`;
            // report them as plain IPv4 so logs and brute force tracking stay consistent.
            unmap_socket_addr(addr),
        ))
    }
}

impl PendingConnection {
    pub async fn establish(self) -> core::result::Result<Connection, (Error, Option<Connection>)> {
        match tokio::time::timeout(HANDSHAKE_TIMEOUT, self.handshake()).await {
            Ok(result) => result,
            Err(_) => Err((
                Error::new(ErrorKind::TimedOut, "Connection handshake timed out"),
                None,
            )),
        }
    }

    async fn handshake(self) -> core::result::Result<Connection, (Error, Option<Connection>)> {
        let Some(tls) = self.encryption.as_ref() else {
            return Ok(Connection::from(self.stream));
        };

        if !is_tls_stream(&self.stream).await.map_err(|e| (e, None))? {
            return Err((
                Error::new(ErrorKind::InvalidData, "Not TLS"),
                Some(Connection::from(self.stream)),
            ));
        }

        tls.accept(self.stream).await.map_err(|e| (e, None))
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
                (0x01..=0x04).contains(&version_minor)) // TLS 1.0 through 1.3
        }
        _ => Ok(false),
    }
}
