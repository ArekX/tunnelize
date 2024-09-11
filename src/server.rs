use std::{collections::HashMap, sync::Arc};

use log::{debug, info};
use tokio::{
    io::{self, AsyncWriteExt, Result},
    net::{TcpListener, TcpStream},
    sync::Mutex,
};

use crate::{
    configuration::ServerConfiguration,
    messages::{read_message, write_message, MessageError, ServerMessage, TunnelMessage},
};

struct Tunnel {
    pub id: u32,
    pub hostname: String,
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

    pub fn remove_tunnel(&mut self, id: u32) {
        self.tunnels.retain(|t| t.id != id);
    }

    pub fn find_by_hostname(&mut self, hostname: String) -> Option<&mut Tunnel> {
        self.tunnels
            .iter_mut()
            .find(|tunnel| tunnel.hostname == hostname)
    }
}

async fn read_until_block(stream: &mut TcpStream) -> String {
    let mut request_buffer = Vec::new();
    loop {
        let mut buffer = [0; 100024];

        stream.readable().await.unwrap();

        match stream.try_read(&mut buffer) {
            Ok(0) => {
                break;
            }
            Ok(read) => {
                request_buffer.extend_from_slice(&buffer[..read]);
                if read < buffer.len() {
                    break;
                }
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                break;
            }
            Err(e) => {
                debug!("Error while reading until block: {:?}", e);
                break;
            }
        }
    }

    match String::from_utf8(request_buffer) {
        Ok(result) => result,
        Err(e) => {
            debug!("Error while converting buffer to string: {:?}", e);
            String::new()
        }
    }
}

fn find_hostname(request: &String) -> Option<String> {
    request
        .lines()
        .find(|line| line.starts_with("Host:"))
        .map(|host_header| host_header.trim_start_matches("Host:").trim().to_string())
}

async fn end_respond_to_client(stream: &mut TcpStream, message: &str) {
    stream.write_all(message.as_bytes()).await.unwrap();
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
        let (mut stream, address) = client.accept().await.unwrap();

        let mut id_counter = client_counter.lock().await;
        (*id_counter) = (*id_counter).wrapping_add(1);
        let client_id = *id_counter;

        println!(
            "Client connected from {}, assigned ID: {}",
            address, client_id
        );

        let initial_request = read_until_block(&mut stream).await;

        let hostname = match find_hostname(&initial_request) {
            Some(hostname) => hostname,
            None => {
                info!("No hostname found in initial request, closing connection.");
                end_respond_to_client(
                    &mut stream,
                    "No hostname found for this request. Cannot resolve to a tunnel. Closing connection.",
                )
                .await;
                continue;
            }
        };
        let mut tunnel_value = tunnel_list.lock().await;
        let tunnel = match tunnel_value.find_by_hostname(hostname.clone()) {
            Some(tunnel) => tunnel,
            None => {
                debug!("No tunnel found for hostname: {}", hostname);
                end_respond_to_client(
                    &mut stream,
                    "No tunnel connected for this hostname. Closing connection.",
                )
                .await;
                continue;
            }
        };

        let id = tunnel.id;

        {
            let mut client_link_map = client_list.lock().await;
            client_link_map.insert(
                client_id,
                Client {
                    stream,
                    initial_request: initial_request.clone(),
                },
            );
        }
        tunnel.connected_client_id = Some(client_id);

        info!(
            "Sending link request to tunnel, for client ID: {}",
            client_id
        );

        match write_message(
            &mut tunnel.stream,
            &ServerMessage::LinkRequest { id: client_id },
        )
        .await
        {
            Ok(_) => {
                debug!("Link request sent to tunnel for client ID: {}", client_id);
            }
            Err(e) => match e {
                MessageError::IoError(err) => {
                    if err.kind() == io::ErrorKind::BrokenPipe
                        || err.kind() == io::ErrorKind::ConnectionReset
                    {
                        debug!("Tunnel disconnected while sending link request.");
                        let mut client = client_list.lock().await.remove(&client_id).unwrap();
                        end_respond_to_client(
                            &mut client.stream,
                            "No tunnel connected for this hostname. Closing connection.",
                        )
                        .await;
                    } else {
                        debug!("Error while sending link request: {:?}", err);
                    }

                    tunnel_value.remove_tunnel(id);
                }
                _ => {
                    debug!("Error while sending link request: {:?}", e);
                    tunnel_value.remove_tunnel(id);
                }
            },
        }
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
                TunnelMessage::Connect { hostname } => {
                    info!(
                        "Tunnel connected for hostname '{}', waiting for client link requests.",
                        hostname
                    );
                    tunnel_list.lock().await.register(Tunnel {
                        id: 0,
                        stream,
                        hostname,
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

                    stream
                        .write_all(client.initial_request.as_bytes())
                        .await
                        .unwrap();

                    match io::copy_bidirectional(&mut client.stream, &mut stream).await {
                        _ => {}
                    }
                }
            }
        });
    }
}

struct Client {
    pub initial_request: String,
    pub stream: TcpStream,
}

type MainClientList = Arc<Mutex<HashMap<u32, Client>>>;
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
