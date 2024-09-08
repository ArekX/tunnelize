use std::error::Error;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn start_proxy() -> Result<(), Box<dyn Error>> {
    let mut server_stream = TcpStream::connect("0.0.0.0:3456").await?;
    let mut proxy_from = TcpStream::connect("0.0.0.0:8000").await?;

    let (mut server_reader, mut server_writer) = server_stream.split();
    let (mut proxy_reader, mut proxy_writer) = proxy_from.split();

    loop {
        tokio::select! {
            result = proxy(&mut server_reader, &mut proxy_writer) => {
                if let Err(e) = result {
                    eprintln!("Error in proxying from server to proxy: {:?}", e);
                }
            },
            result = proxy(&mut proxy_reader, &mut server_writer) => {
                if let Err(e) = result {
                    eprintln!("Error in proxying from proxy to server: {:?}", e);
                }
            },
        }
    }

    Ok(())
}

async fn proxy(
    from_socket: &mut (impl AsyncReadExt + Unpin),
    to_socket: &mut (impl AsyncWriteExt + Unpin),
) -> Result<(), Box<dyn Error>> {
    let mut buffer = [0; 1024];

    loop {
        let n = from_socket.read(&mut buffer).await?;
        if n == 0 {
            break;
        }
        to_socket.write_all(&buffer[..n]).await?;
    }

    Ok(())
}
