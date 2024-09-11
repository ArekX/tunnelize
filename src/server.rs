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

struct Tunnel {
    pub connected_client_id: Option<u32>,
    pub stream: TcpStream,
}

struct TunnelList {
    tunnels: Vec<Tunnel>,
}

impl TunnelList {
    pub fn new() -> Self {
        TunnelList {
            tunnels: Vec::new(),
        }
    }

    pub fn register(&mut self, tunnel: Tunnel) {
        self.tunnels.push(tunnel);
    }

    pub fn get(&mut self) -> &mut Tunnel {
        self.tunnels.get_mut(0).unwrap()
    }
}

async fn listen_to_client(
    client_address: String,
    client_list: MainClientList,
    tunnel_list: MainTunnelList,
) -> Result<()> {
    let client_counter = Arc::new(Mutex::new(0 as u32));

    let client = TcpListener::bind(client_address.clone()).await.unwrap();

    info!("Listening to client connections on {}", client_address);

    loop {
        let (stream, address) = client.accept().await.unwrap();

        let mut id_counter = client_counter.lock().await;
        (*id_counter) = (*id_counter).wrapping_add(1);
        let client_id = *id_counter;

        println!(
            "Client connected from {}, assigned ID: {}",
            address, client_id
        );

        let mut client_link_map = client_list.lock().await;
        client_link_map.insert(client_id, stream);

        let mut tunnel_value = tunnel_list.lock().await;
        let tunnel = tunnel_value.get();

        tunnel.connected_client_id = Some(client_id);

        info!(
            "Sending link request to tunnel, for client ID: {}",
            client_id
        );
        write_message(
            &mut tunnel.stream,
            &ServerMessage::LinkRequest { id: client_id },
        )
        .await
        .unwrap();
    }
}

async fn listen_to_tunnel(
    tunnel_address: String,
    client_list: MainClientList,
    tunnel_list: MainTunnelList,
) -> Result<()> {
    let link = TcpListener::bind(tunnel_address.clone()).await.unwrap();

    info!("Listening to tunnel connections on {}", tunnel_address);

    loop {
        let (mut stream, address) = link.accept().await.unwrap();

        info!("Link established with {}", address);

        stream.readable().await.unwrap();

        let tunnel_list = tunnel_list.clone();
        let client_list = client_list.clone();

        tokio::spawn(async move {
            let message: TunnelMessage = if let Ok(m) = read_message(&mut stream).await {
                m
            } else {
                return;
            };

            match message {
                TunnelMessage::Connect => {
                    info!("Tunnel connected, waiting for link request.");
                    tunnel_list.lock().await.register(Tunnel {
                        stream,
                        connected_client_id: None,
                    });
                }
                TunnelMessage::LinkAccept { id } => {
                    info!("Link accepted for client ID: {}", id);
                    let mut client = {
                        let mut client_list = client_list.lock().await;

                        if !client_list.contains_key(&id) {
                            return;
                        }

                        client_list.remove_entry(&id).unwrap().1
                    };

                    match io::copy_bidirectional(&mut client, &mut stream).await {
                        _ => {}
                    }
                }
            }
        });
    }
}

type MainClientList = Arc<Mutex<HashMap<u32, TcpStream>>>;
type MainTunnelList = Arc<Mutex<TunnelList>>;

pub async fn start_server(config: ServerConfiguration) -> Result<()> {
    let main_client_list: MainClientList = Arc::new(Mutex::new(HashMap::new()));
    let main_tunnel_list: MainTunnelList = Arc::new(Mutex::new(TunnelList::new()));

    let tunnel_client_list = main_client_list.clone();
    let tunnel_list = main_tunnel_list.clone();
    let tunnel_listener = tokio::spawn(async move {
        listen_to_tunnel(
            config.tunnel_address.clone(),
            tunnel_client_list,
            tunnel_list,
        )
        .await
        .unwrap();
    });

    let client_list = main_client_list.clone();
    let tunnel_list = main_tunnel_list.clone();
    let client_listener = tokio::spawn(async move {
        listen_to_client(config.client_address.clone(), client_list, tunnel_list)
            .await
            .unwrap();
    });

    info!("Server startup finished.");
    tunnel_listener.await.unwrap();
    client_listener.await.unwrap();

    Ok(())
}
