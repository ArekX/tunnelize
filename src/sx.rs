use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt, Interest, Result};
use tokio::net::{TcpListener, TcpSocket, TcpStream};
use tokio::sync::{Mutex, MutexGuard};

pub async fn server() -> Result<()> {
    let tunnel = TcpListener::bind("0.0.0.0:3456").await?;
    let client = TcpListener::bind("0.0.0.0:3457").await?;

    let tunnel_client: Arc<Mutex<Option<TcpStream>>> = Arc::new(Mutex::new(None));

    loop {
        tokio::select! {
            Ok((tunnel_stream, address)) = tunnel.accept() => {
                let mut tunnel_client = tunnel_client.lock().await;
                println!("Tunnel established with {}", address);
                *tunnel_client = Some(tunnel_stream);
            },
            Ok((mut user_stream, user_addr)) = client.accept() => {
                let tunnel = tunnel_client.clone();
                println!("\n\n\n\n\n\nClient connected from {}", user_addr);
                let mut tunnel_value = tunnel.lock().await;
                    let client = tunnel_value.as_mut().unwrap();

                    user_stream.readable().await.unwrap();

                    println!("Server: Copying data to tunnel");
                    forward(&mut user_stream, client).await.unwrap();

                    client.readable().await.unwrap();

                    println!("Server: Copying data to user");
                    forward(client, &mut user_stream).await.unwrap();

                    println!("Server: Copy complete");

            },
        }
    }
}

pub async fn client() -> Result<()> {
    let mut server = TcpStream::connect("0.0.0.0:3456").await?;

    loop {
        println!("Waiting for request.");
        server.readable().await?;

        println!("Request received.");

        let mut proxy = TcpStream::connect("0.0.0.0:8000").await.unwrap();

        println!("Proxy Copying data to proxy");
        forward(&mut server, &mut proxy).await.unwrap();

        println!("Proxy Copying data to server");
        forward(&mut proxy, &mut server).await.unwrap();

        println!("Proxy Copy complete");
    }
}

const BUFFER_SIZE: usize = 8 * 1024;

#[derive(Debug)]
enum ProcessRes {
    // Begin,
    End,
    WaitingResponse,
    Done,
}

async fn forward(from: &mut TcpStream, to: &mut TcpStream) -> Result<ProcessRes> {
    let mut buffer = [0; BUFFER_SIZE];

    loop {
        from.readable().await?;
        to.writable().await?;

        match from.try_read(&mut buffer) {
            Ok(0) => {
                return Ok(ProcessRes::Done);
            }
            Ok(n) => {
                to.write_all(&buffer[..n]).await?;
            }
            Err(e) if e.kind() == tokio::io::ErrorKind::WouldBlock => {
                println!("Blocked, waiting for response");
                return Ok(ProcessRes::Done);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
}

pub async fn start() -> tokio::io::Result<()> {
    // Spawn server and client handlers
    tokio::spawn(async move {
        server().await.unwrap();
    });

    tokio::spawn(async move {
        client().await.unwrap();
    });

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        tokio::task::yield_now().await;
    }
}
