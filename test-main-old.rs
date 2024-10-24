use rustls::server::{ClientHello, ResolvesServerCert};
use rustls::{ClientConfig, RootCertStore, ServerConfig};
use std::io;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::{TlsAcceptor, TlsConnector};

#[derive(Debug)]
struct SNIExtractor;

impl ResolvesServerCert for SNIExtractor {
    fn resolve(&self, client_hello: ClientHello) -> Option<Arc<rustls::sign::CertifiedKey>> {
        if let Some(server_name) = client_hello.server_name() {
            println!("Server received SNI idemo niis!: {}", server_name);
        }
        None
    }
}

async fn run_server() -> io::Result<()> {
    let server_config = ServerConfig::builder()
        .with_no_client_auth()
        .with_cert_resolver(Arc::new(SNIExtractor));

    let acceptor = TlsAcceptor::from(Arc::new(server_config));
    let listener = TcpListener::bind("127.0.0.1:8443").await?;

    println!("Server listening on :8443");

    while let Ok((stream, _)) = listener.accept().await {
        let acceptor = acceptor.clone();

        tokio::spawn(async move {
            match acceptor.accept(stream).await {
                Ok(_) => println!("Handshake attempted"),
                Err(e) => eprintln!("Handshake error: {}", e),
            }
        });
    }

    Ok(())
}

async fn run_client() -> io::Result<()> {
    // Create client configuration
    let root_store = RootCertStore::empty();
    let client_config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let connector = TlsConnector::from(Arc::new(client_config));

    // Connect to the server
    let domain = "example.com".try_into().unwrap();
    let stream = TcpStream::connect("127.0.0.1:8443").await?;

    println!("Client connecting with SNI: example.com");

    // Attempt TLS handshake
    match connector.connect(domain, stream).await {
        Ok(_) => println!("Client handshake completed"),
        Err(e) => println!("Client handshake failed (expected): {}", e),
    }

    Ok(())
}

#[tokio::main]
async fn main() -> io::Result<()> {
    // Spawn the server task
    let server = tokio::spawn(run_server());

    // Wait a bit for the server to start
    tokio::time::sleep(Duration::from_secs(1)).await;

    // Run the client
    run_client().await?;

    // Wait for server task (optional)
    if let Err(e) = server.await.unwrap() {
        eprintln!("Server error: {}", e);
    }

    Ok(())
}
