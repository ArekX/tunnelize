use std::error::Error;
use std::net::SocketAddr;
use tokio::io::{self};
use tokio::net::{TcpListener, TcpStream};

pub async fn start_proxy() -> Result<(), Box<dyn Error>> {
    let bind_addr: SocketAddr = "127.0.0.1:9080".parse()?;
    let dest_addr: SocketAddr = "127.0.0.1:8000".parse()?;

    let listener = TcpListener::bind(bind_addr).await?;
    println!("Proxy listening on {}", bind_addr);

    /*
    Add routing per app route or per host
    client and server should have json configuration files
    client connects to the server, authenticates and then
    sends the configuration file to the server
    set keepalive to 30 seconds

    */

    loop {
        let (client_socket, _) = listener.accept().await?;
        tokio::spawn(async move {
            if let Err(e) = proxy(client_socket, dest_addr).await {
                eprintln!("Error: {}", e);
            }
        });
    }
}

async fn proxy(mut client_socket: TcpStream, dest_addr: SocketAddr) -> Result<(), Box<dyn Error>> {
    let mut dest_socket = TcpStream::connect(dest_addr).await?;
    let (mut client_reader, mut client_writer) = client_socket.split();
    let (mut dest_reader, mut dest_writer) = dest_socket.split();

    let client_to_dest = io::copy(&mut client_reader, &mut dest_writer);
    let dest_to_client = io::copy(&mut dest_reader, &mut client_writer);

    tokio::try_join!(client_to_dest, dest_to_client)?;
    Ok(())
}
