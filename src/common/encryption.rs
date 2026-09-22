use rustls::{
    pki_types::{pem::PemObject, CertificateDer, PrivateKeyDer, ServerName},
    ClientConfig, RootCertStore,
};

use rustls_native_certs::load_native_certs;

use std::{
    collections::HashMap,
    io::{Error, ErrorKind},
    sync::{Arc, Mutex, OnceLock},
};

use tokio_rustls::{TlsAcceptor, TlsConnector};

use super::connection::Connection;
use tokio::{io::Result, net::TcpStream};

pub struct ServerTlsEncryption {
    acceptor: TlsAcceptor,
}

impl ServerTlsEncryption {
    pub async fn new(cert_path: &str, key_path: &str, alpn_protocols: Vec<Vec<u8>>) -> Self {
        let cert_reader =
            CertificateDer::pem_file_iter(cert_path).expect("Failed to read Server cert");

        let key = PrivateKeyDer::from_pem_file(key_path).expect("Failed to read Server key");

        let certs: Vec<CertificateDer> = cert_reader
            .flatten()
            .collect();

        let mut config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .expect("Failed to create Server config");

        config.alpn_protocols = alpn_protocols;

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

/// Client TLS configurations by CA path. Building one means reading the CA file
/// or the whole OS certificate store, which is slow (hundreds of milliseconds on
/// macOS) and blocking, so it is done once per process instead of once per link.
static CLIENT_CONFIGS: OnceLock<Mutex<HashMap<Option<String>, Arc<ClientConfig>>>> =
    OnceLock::new();

impl ClientTlsEncryption {
    pub async fn new(ca_path: Option<String>) -> Self {
        let connector = TlsConnector::from(Self::client_config(ca_path).await);

        ClientTlsEncryption { connector }
    }

    async fn client_config(ca_path: Option<String>) -> Arc<ClientConfig> {
        let cache = CLIENT_CONFIGS.get_or_init(|| Mutex::new(HashMap::new()));

        if let Some(config) = cache.lock().expect("TLS config cache poisoned").get(&ca_path) {
            return config.clone();
        }

        let config = {
            let ca_path = ca_path.clone();

            tokio::task::spawn_blocking(move || {
                Arc::new(
                    ClientConfig::builder()
                        .with_root_certificates(Self::resolve_client_root_cert_store(ca_path))
                        .with_no_client_auth(),
                )
            })
            .await
            .expect("Failed to build TLS client configuration")
        };

        cache
            .lock()
            .expect("TLS config cache poisoned")
            .entry(ca_path)
            .or_insert(config)
            .clone()
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
                    .flatten()
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
