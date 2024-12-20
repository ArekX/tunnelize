use rustls::{
    pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer, ServerName},
    ClientConfig, RootCertStore,
};

use rustls_native_certs::load_native_certs;

use std::{
    io::{Error, ErrorKind},
    sync::Arc,
};

use tokio_rustls::{TlsAcceptor, TlsConnector};

use super::connection::Connection;
use tokio::{io::Result, net::TcpStream};

pub struct ServerTlsEncryption {
    acceptor: TlsAcceptor,
}

impl ServerTlsEncryption {
    pub async fn new(cert_path: &str, key_path: &str) -> Self {
        let cert_reader =
            CertificateDer::pem_file_iter(cert_path).expect("Failed to read Server cert");

        let key = PrivateKeyDer::from_pem_file(key_path).expect("Failed to read Server key");

        let certs: Vec<CertificateDer> = cert_reader
            .filter(|i| i.is_ok())
            .map(|i| i.unwrap())
            .collect();

        let config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .expect("Failed to create Server config");

        let acceptor = TlsAcceptor::from(Arc::new(config));

        ServerTlsEncryption { acceptor }
    }

    pub async fn accept(&self, stream: TcpStream) -> Result<Connection> {
        let stream = self.acceptor.accept(stream).await?;

        Ok(Connection::from(stream))
    }
}

pub struct ClientTlsEncryption {
    connector: TlsConnector,
}

impl ClientTlsEncryption {
    pub async fn new(ca_path: Option<String>) -> Self {
        let client_config = ClientConfig::builder()
            .with_root_certificates(Self::resolve_client_root_cert_store(ca_path))
            .with_no_client_auth();

        let connector = TlsConnector::from(Arc::new(client_config));

        ClientTlsEncryption { connector }
    }

    pub async fn connect(&self, stream: TcpStream, domain: &str) -> Result<Connection> {
        let Ok(domain) = ServerName::try_from(domain.to_owned()) else {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Failed to parse domain",
            ));
        };
        let stream = self.connector.connect(domain, stream).await?;

        Ok(Connection::from(stream))
    }

    fn resolve_client_root_cert_store(ca_path: Option<String>) -> RootCertStore {
        let mut root_store = RootCertStore::empty();

        match ca_path {
            Some(ca_path) => {
                let cert_reader =
                    CertificateDer::pem_file_iter(ca_path).expect("Failed to read CA certificate");
                let certs: Vec<CertificateDer> = cert_reader
                    .filter(|i| i.is_ok())
                    .map(|i| i.unwrap())
                    .collect();

                for cert in certs {
                    root_store
                        .add(cert)
                        .expect("Failed to add CA certificate to root store");
                }
            }
            None => {
                let native_certs =
                    load_native_certs().expect("Failed to load native OS certificates");

                for cert in native_certs {
                    root_store
                        .add(cert)
                        .expect("Failed to add Native certificate to root store");
                }
            }
        }

        root_store
    }
}
