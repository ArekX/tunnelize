use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt, Result},
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

const BUFFER_SIZE: usize = 8 * 1024;

#[derive(Debug, Serialize, Deserialize)]
enum Message {
    LinkRequest { id: u32 },
    LinkAccept { id: u32 },
    LinkEnded { id: u32 },
}

struct TunnelLink {
    pub client: TcpStream,
    pub tunnel: Option<TcpStream>,
}

pub async fn start_server() -> Result<()> {
    let client_counter = Arc::new(Mutex::new(0));
    let tunnel_client: Arc<Mutex<Option<TcpStream>>> = Arc::new(Mutex::new(None));
    let client_link_map: Arc<Mutex<HashMap<u32, TunnelLink>>> =
        Arc::new(Mutex::new(HashMap::new()));

    let link_tunnel_client = tunnel_client.clone();

    tokio::spawn(async move {
        let tunnel = TcpListener::bind("0.0.0.0:3455").await.unwrap();
        loop {
            let (stream, address) = tunnel.accept().await.unwrap();
            let mut new_client = link_tunnel_client.lock().await;
            println!("Tunnel established with {}", address);
            *new_client = Some(stream);
        }
    });

    let cloned_tunnel = tunnel_client.clone();
    let cloned_link_map = client_link_map.clone();

    tokio::spawn(async move {
        let client = TcpListener::bind("0.0.0.0:3457").await.unwrap();

        loop {
            let (stream, address) = client.accept().await.unwrap();
            println!("Client connected from {}", address);
            let mut id_counter = client_counter.lock().await;
            *id_counter += 1;
            let id = *id_counter;

            let mut client_link_map = cloned_link_map.lock().await;
            client_link_map.insert(
                id,
                TunnelLink {
                    client: stream,
                    tunnel: None,
                },
            );

            let mut tunnel_value = cloned_tunnel.lock().await;
            let client = tunnel_value.as_mut().unwrap();

            let message = serde_json::to_string(&Message::LinkRequest { id }).unwrap();

            client.write_all(message.as_bytes()).await.unwrap();
        }
    });

    let link_client_map = client_link_map.clone();

    tokio::spawn(async move {
        let link = TcpListener::bind("0.0.0.0:3456").await.unwrap();

        loop {
            let (mut stream, address) = link.accept().await.unwrap();

            println!("Link established with {}", address);

            stream.readable().await.unwrap();

            let mut buffer = [0; BUFFER_SIZE];
            let n = stream.read(&mut buffer).await.unwrap();

            let message: Message = serde_json::from_slice(&buffer[..n]).unwrap();

            match message {
                Message::LinkAccept { id } => {
                    let mut client_link_map = link_client_map.lock().await;
                    let link = client_link_map.get_mut(&id).unwrap();
                    io::copy_bidirectional(&mut link.client, &mut stream)
                        .await
                        .unwrap();

                    client_link_map.remove(&id);
                }
                _ => {}
            }
        }
    });

    println!("Server started");

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(300)).await;
    }
}

pub async fn start_client() -> Result<()> {
    let mut server = TcpStream::connect("54.87.75.143:3455").await?;

    loop {
        println!("Waiting for request.");
        server.readable().await?;

        println!("Request received.");
        let mut buffer = [0; BUFFER_SIZE];
        let n = server.read(&mut buffer).await.unwrap();

        let message: Message = serde_json::from_slice(&buffer[..n]).unwrap();

        match message {
            Message::LinkRequest { id } => {
                let mut tunnel = TcpStream::connect("54.87.75.143:3456").await.unwrap();
                let mut proxy = TcpStream::connect("0.0.0.0:8000").await.unwrap();

                tunnel
                    .write_all(
                        serde_json::to_string(&Message::LinkAccept { id })
                            .unwrap()
                            .as_bytes(),
                    )
                    .await
                    .unwrap();

                io::copy_bidirectional(&mut tunnel, &mut proxy)
                    .await
                    .unwrap();
            }
            _ => {}
        }
    }
}
