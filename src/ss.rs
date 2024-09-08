use tokio::io::{copy, split};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;

async fn handle_client_to_server(client: TcpStream, server: TcpStream) -> tokio::io::Result<()> {
    let (mut client_read, mut client_write) = split(client);
    let (mut server_read, mut server_write) = split(server);

    let client_to_server = copy(&mut client_read, &mut server_write);
    let server_to_client = copy(&mut server_read, &mut client_write);

    // Forward traffic between client and server (port 3456 and 3457)
    tokio::join!(client_to_server, server_to_client,).0?;

    Ok(())
}

async fn handle_server_to_client(
    user_stream: TcpStream,
    client_tx: mpsc::Sender<TcpStream>,
) -> tokio::io::Result<()> {
    // Forward request to client via channel
    client_tx.send(user_stream).await.unwrap();
    Ok(())
}

async fn run_server(client_tx: mpsc::Sender<TcpStream>) -> tokio::io::Result<()> {
    // Server listens on port 3457 for user connections
    let listener = TcpListener::bind("0.0.0.0:3457").await?;

    loop {
        let (user_stream, _) = listener.accept().await?;
        let client_tx = client_tx.clone();
        tokio::spawn(async move {
            handle_server_to_client(user_stream, client_tx)
                .await
                .unwrap();
        });
    }
}

async fn run_client() -> tokio::io::Result<()> {
    // Client listens on port 3456 for server connections
    let listener = TcpListener::bind("0.0.0.0:3456").await?;

    loop {
        let (server_stream, _) = listener.accept().await?;
        let local_service = TcpStream::connect("127.0.0.1:8000").await?;

        // Forward traffic between local service (8000) and server (3456)
        tokio::spawn(async move {
            handle_client_to_server(server_stream, local_service)
                .await
                .unwrap();
        });
    }
}

pub async fn start() -> tokio::io::Result<()> {
    let (client_tx, mut client_rx) = mpsc::channel::<TcpStream>(32);

    // Spawn server and client handlers
    tokio::spawn(async move {
        run_server(client_tx).await.unwrap();
    });

    tokio::spawn(async move {
        run_client().await.unwrap();
    });

    // Process requests from the server to the client
    while let Some(user_stream) = client_rx.recv().await {
        let client_stream = TcpStream::connect("127.0.0.1:3456").await.unwrap();
        tokio::spawn(async move {
            handle_client_to_server(client_stream, user_stream)
                .await
                .unwrap();
        });
    }

    Ok(())
}
