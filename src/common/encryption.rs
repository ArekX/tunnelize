use rustls::{
    pki_types::{pem::PemObject, CertificateDer, IpAddr, Ipv4Addr, PrivateKeyDer, ServerName},
    ClientConfig, RootCertStore,
};

use std::sync::Arc;

use tokio_rustls::{TlsAcceptor, TlsConnector};

use super::connection::ConnectionStream;
use tokio::{io::Result, net::TcpStream};

pub struct ServerTlsEncryption {
    acceptor: TlsAcceptor,
}

impl ServerTlsEncryption {
    pub async fn new(cert_path: &str, key_path: &str) -> Self {
        let cert_reader = CertificateDer::pem_file_iter(cert_path).unwrap();
        let certs: Vec<CertificateDer> = cert_reader.map(|i| i.unwrap()).collect();
        let key = PrivateKeyDer::from_pem_file(key_path).unwrap();

        let config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .unwrap();

        let acceptor = TlsAcceptor::from(Arc::new(config));

        ServerTlsEncryption { acceptor }
    }

    pub async fn accept(&self, stream: TcpStream) -> Result<ConnectionStream> {
        let stream = self.acceptor.accept(stream).await?;

        Ok(ConnectionStream::from(stream))
    }
}

pub struct ClientTlsEncryption {
    connector: TlsConnector,
}

impl ClientTlsEncryption {
    pub async fn new(cert_path: String) -> Self {
        let mut root_store = RootCertStore::empty();

        let cert_reader = CertificateDer::pem_file_iter(cert_path).unwrap();
        let certs: Vec<CertificateDer> = cert_reader.map(|i| i.unwrap()).collect();

        for cert in certs {
            root_store
                .add(cert)
                .expect("Failed to add certificate to root store");
        }

        let client_config = ClientConfig::builder()
            .with_root_certificates(root_store)
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
}
