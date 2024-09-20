use std::time::Duration;

use log::debug;
use tokio::{io, net::TcpStream, time::timeout};

pub struct ResolvedClient {
    pub initial_request: String,
    pub resolved_host: Option<String>,
}

pub async fn resolve_http_client(stream: &mut TcpStream) -> ResolvedClient {
    let request = read_until_block(stream).await;
    let hostname = find_hostname(&request);

    ResolvedClient {
        initial_request: request,
        resolved_host: hostname,
    }
}

async fn read_until_block(stream: &mut TcpStream) -> String {
    let mut request_buffer = Vec::new();
    let duration = Duration::from_secs(5);
    loop {
        debug!("Waiting tcp stream to be readable...");
        match timeout(duration, stream.readable()).await {
            Ok(_) => {}
            Err(_) => {
                debug!("Timeout while waiting for client stream to be readable.");
                break;
            }
        }

        let mut buffer = [0; 100024];

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
        .map(|host_header| {
            host_header
                .trim_start_matches("Host:")
                .trim()
                .split_once(':')
                .map_or_else(|| host_header.to_string(), |(host, _)| host.to_string())
        })
}
