use std::{
    io::{self, Error, ErrorKind},
    net::SocketAddr,
    sync::Arc,
};

use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, TcpStream},
};
use tokio::{
    io::{AsyncReadExt, Interest},
    sync::Mutex,
};

struct Tunneler {
    pub address: SocketAddr,
    pub socket: TcpStream,
}

pub async fn start_server() -> Result<(), Error> {
    let tunneler: Arc<Mutex<Vec<Tunneler>>> = Arc::new(Mutex::new(vec![]));

    let tunnel_listener = run_tunnel_listener(tunneler.clone());
    let pass_listener = run_pass_listener(tunneler.clone());

    tokio::try_join!(tunnel_listener, pass_listener)?;

    Ok(())
}

async fn run_tunnel_listener(manager: Arc<Mutex<Vec<Tunneler>>>) -> Result<(), Error> {
    let listener = TcpListener::bind("0.0.0.0:3456").await?;

    println!("Tunnel listener started on 3456");

    loop {
        let (socket, address) = listener.accept().await?;

        let db = manager.clone();
        tokio::spawn(async move {
            println!("Tunnel established with {}", address);
            db.lock().await.push(Tunneler { address, socket });
        });
    }
}

async fn run_pass_listener(manager: Arc<Mutex<Vec<Tunneler>>>) -> Result<(), Error> {
    let listener = TcpListener::bind("0.0.0.0:3457").await?;

    println!("Pass listener started on 3457");

    loop {
        let (socket, _) = listener.accept().await?;

        let db = manager.clone();

        tokio::spawn(async move {
            process(socket, db).await.unwrap();
        });
    }
}

async fn process(mut socket: TcpStream, db: Arc<Mutex<Vec<Tunneler>>>) -> Result<(), Error> {
    let mut vb = db.lock().await;

    let first = vb.first_mut();

    if let None = first {
        socket
            .write_all(b"Waiting for connection to be established.")
            .await?;

        return Ok(());
    }

    let actual = first.unwrap();

    proxy(&mut socket, &mut actual.socket).await?;
    proxy(&mut actual.socket, &mut socket).await?;

    Ok(())
}

async fn proxy(
    from_socket: &mut TcpStream,
    to_socket: &mut TcpStream,
) -> Result<(), std::io::Error> {
    let mut buffer = [0; 8 * 1024];

    loop {
        let n = from_socket.read(&mut buffer).await?;

        if n == 0 {
            break;
        }

        to_socket.write_all(&buffer[..n]).await?;
    }

    Ok(())
}
