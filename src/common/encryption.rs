use rustls::{
    pki_types::{pem::PemObject, CertificateDer, IpAddr, Ipv4Addr, PrivateKeyDer, ServerName},
    ClientConfig, RootCertStore,
};

use rustls_native_certs::load_native_certs;

use std::sync::Arc;

use tokio_rustls::{TlsAcceptor, TlsConnector};

use super::connection::ConnectionStream;
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

    pub async fn accept(&self, stream: TcpStream) -> Result<ConnectionStream> {
        let stream = self.acceptor.accept(stream).await?;

        Ok(ConnectionStream::from(stream))
    }
}

pub enum ClientEncryptionType {
    CustomTls { ca_cert_path: String },
    NativeTls,
}

pub struct ClientTlsEncryption {
    connector: TlsConnector,
}

impl ClientTlsEncryption {
    pub async fn new(encryption_type: ClientEncryptionType) -> Self {
        let client_config = ClientConfig::builder()
            .with_root_certificates(Self::resolve_client_root_cert_store(encryption_type))
            .with_no_client_auth();

        let connector = TlsConnector::from(Arc::new(client_config));

        ClientTlsEncryption { connector }
    }

    pub async fn connect(&self, stream: TcpStream, domain: String) -> Result<ConnectionStream> {
        let domain = match domain.try_into() {
            Ok(domain) => domain,
            Err(e) => {
                log::error!("Failed to convert domain: {}", e);
                // TODO: Correctly check if the domain is an IP address
                ServerName::IpAddress(IpAddr::V4(Ipv4Addr::try_from("127.0.0.1").unwrap()))
            }
        };
        let stream = self.connector.connect(domain, stream).await?;

        Ok(ConnectionStream::from(stream))
    }

    fn resolve_client_root_cert_store(encryption_type: ClientEncryptionType) -> RootCertStore {
        let mut root_store = RootCertStore::empty();

        match encryption_type {
            ClientEncryptionType::CustomTls { ca_cert_path } => {
                let cert_reader =
                    CertificateDer::pem_file_iter(ca_cert_path).expect("Failed to read CA cert");
                let certs: Vec<CertificateDer> = cert_reader
                    .filter(|i| i.is_ok())
                    .map(|i| i.unwrap())
                    .collect();

                for cert in certs {
                    root_store
                        .add(cert)
                        .expect("Failed to add certificate to root store");
                }
            }
            ClientEncryptionType::NativeTls => {
                let native_certs = load_native_certs().expect("Failed to load native certs");

                for cert in native_certs {
                    root_store
                        .add(cert)
                        .expect("Failed to add certificate to root store");
                }
            }
        }

        root_store
    }
}
