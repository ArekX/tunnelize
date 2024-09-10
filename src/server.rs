use std::{collections::HashMap, sync::Arc};

use log::info;
use tokio::{
    io::{self, Result},
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

use crate::{
    configuration::ServerConfiguration,
    messages::{read_message, write_message, ServerMessage, TunnelMessage},
};

struct TunnelLink {
    pub client: TcpStream,
}

pub async fn start_server(config: ServerConfiguration) -> Result<()> {
    let client_counter = Arc::new(Mutex::new(0));
    let tunnel_client: Arc<Mutex<Option<TcpStream>>> = Arc::new(Mutex::new(None));
    let client_link_map: Arc<Mutex<HashMap<u32, TunnelLink>>> =
        Arc::new(Mutex::new(HashMap::new()));

    let cloned_tunnel = tunnel_client.clone();
    let cloned_link_map = client_link_map.clone();

    tokio::spawn(async move {
        let client = TcpListener::bind(config.client_address.clone())
            .await
            .unwrap();

        info!(
            "Listening to client connections on {}",
            config.client_address
        );

        loop {
            let (stream, address) = client.accept().await.unwrap();
            println!("Client connected from {}", address);
            let mut id_counter = client_counter.lock().await;
            *id_counter += 1;
            let id = *id_counter;

            let mut client_link_map = cloned_link_map.lock().await;
            client_link_map.insert(id, TunnelLink { client: stream });

            let mut tunnel_value = cloned_tunnel.lock().await;
            let client = tunnel_value.as_mut().unwrap();

            write_message(client, &ServerMessage::LinkRequest { id })
                .await
                .unwrap();
        }
    });

    let link_client_map = client_link_map.clone();

    let link_tunnel_client = tunnel_client.clone();

    tokio::spawn(async move {
        let link = TcpListener::bind(config.tunnel_address.clone())
            .await
            .unwrap();

        info!(
            "Listening to tunnel connections on {}",
            config.tunnel_address
        );

        loop {
            let (mut stream, address) = link.accept().await.unwrap();

            info!("Link established with {}", address);

            stream.readable().await.unwrap();

            let link_client_map_clone = link_client_map.clone();
            let link_client_tunnel_clone = link_tunnel_client.clone();

            tokio::spawn(async move {
                let message: TunnelMessage = if let Ok(m) = read_message(&mut stream).await {
                    m
                } else {
                    return;
                };

                match message {
                    TunnelMessage::Connect => {
                        let mut new_client = link_client_tunnel_clone.lock().await;
                        info!("Tunnel established with {}", address);
                        *new_client = Some(stream);
                    }
                    TunnelMessage::LinkAccept { id } => {
                        let mut link = {
                            let mut client_link_map = link_client_map_clone.lock().await;
                            let (_, link) = client_link_map.remove_entry(&id).unwrap();
                            link
                        };

                        match io::copy_bidirectional(&mut link.client, &mut stream).await {
                            _ => {}
                        }
                    }
                }
            });
        }
    });

    info!("Server started");

    loop {
        tokio::time::sleep(std::time::Duration::from_secs(300)).await;
    }
}
