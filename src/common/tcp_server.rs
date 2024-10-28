use std::net::SocketAddr;

use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;

use super::{connection::ConnectionStream, encryption::ServerTlsEncryption};
use tokio::io::Result;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ServerEncryption {
    None,
    Tls { cert: String, key: String },
}

pub struct TcpServer {
    encryption: Option<ServerTlsEncryption>,
    listener: TcpListener,
}

impl TcpServer {
    pub async fn new(address: String, port: u16, encryption: ServerEncryption) -> Result<Self> {
        Ok(TcpServer {
            encryption: match encryption {
                ServerEncryption::None => None,
                ServerEncryption::Tls { cert, key } => {
                    Some(ServerTlsEncryption::new(&cert, &key).await)
                }
            },
            listener: TcpListener::bind(format!("{}:{}", address, port)).await?,
        })
    }

    pub async fn listen_for_connection(&self) -> Result<(ConnectionStream, SocketAddr)> {
        let (stream, addr) = self.listener.accept().await?;

        match self.encryption {
            Some(ref tls) => Ok((tls.accept(stream).await?, addr)),
            None => Ok((ConnectionStream::from(stream), addr)),
        }
    }
}
