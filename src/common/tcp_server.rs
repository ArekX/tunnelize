use rustls::pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer};

use std::{net::SocketAddr, sync::Arc};

use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

use super::connection::ConnectionStream;
use tokio::io::Result;

pub enum ServerEncryption {
    None,
    Tls { cert: String, key: String },
}

struct TlsEncryption {
    acceptor: TlsAcceptor,
}

impl TlsEncryption {
    pub async fn new(cert_path: &str, key_path: &str) -> Self {
        let cert_reader = CertificateDer::pem_file_iter(cert_path).unwrap();
        let certs: Vec<CertificateDer> = cert_reader.map(|i| i.unwrap()).collect();
        let key = PrivateKeyDer::from_pem_file(key_path).unwrap();

        let config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .unwrap();

        let acceptor = TlsAcceptor::from(Arc::new(config));

        TlsEncryption { acceptor }
    }

    pub async fn accept(&self, stream: tokio::net::TcpStream) -> Result<ConnectionStream> {
        let stream = self.acceptor.accept(stream).await?;

        Ok(ConnectionStream::from(stream))
    }
}

pub struct TcpServer {
    encryption: Option<TlsEncryption>,
    listener: TcpListener,
}

impl TcpServer {
    pub async fn new(address: String, port: u16, encryption: ServerEncryption) -> Result<Self> {
        Ok(TcpServer {
            encryption: match encryption {
                ServerEncryption::None => None,
                ServerEncryption::Tls { cert, key } => Some(TlsEncryption::new(&cert, &key).await),
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
