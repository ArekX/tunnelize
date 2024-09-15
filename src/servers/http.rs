use log::{debug, info};
use serde::{Deserialize, Serialize};
use tokio::{
    io::{self, AsyncWriteExt, Result},
    net::{TcpListener, TcpStream},
};

use crate::{
    data::{
        client::{Client, MainClientList},
        tunnel::MainTunnelList,
    },
    messages::{write_message, MessageError, ServerMessage},
};

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

pub async fn start_http_server(
    config: HttpServer,
    client_list: MainClientList,
    tunnel_list: MainTunnelList,
) -> Result<()> {
    let mut client_counter: u32 = 0;
    let client_port = config.port;

    let client = TcpListener::bind(format!("0.0.0.0:{}", client_port))
        .await
        .unwrap();

    info!("Listening to client connections on 0.0.0.0:{}", client_port);

    loop {
        let (mut stream, address) = client.accept().await.unwrap();

        client_counter = client_counter.wrapping_add(1);
        let client_id = client_counter;

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

#[derive(Debug, Serialize, Deserialize)]
pub struct HttpServer {
    pub port: u16,
    pub auth_key: Option<String>,
}
