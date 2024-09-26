use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine as _,
};
use std::time::Duration;

use log::debug;
use tokio::{
    io::{self, AsyncWriteExt},
    net::TcpStream,
    time::timeout,
};

pub struct ResolvedClient {
    pub initial_request: String,
    pub is_authorized: Option<bool>,
    pub resolved_host: Option<String>,
}

pub async fn read_http_client_request(stream: &mut TcpStream) -> String {
    read_until_block(stream).await
}

pub async fn resolve_http_hostname(request: &String) -> Option<String> {
    find_header_value(request, "Host")
}

pub async fn is_authorized(request: &String, username: String, password: String) -> bool {
    let authorization = find_header_value(&request, "Authorization");

    if let Some(authorization) = authorization {
        let expected_authorization =
            general_purpose::STANDARD.encode(format!("{}:{}", username, password));

        if let Some(auth_value) = authorization.split_whitespace().last() {
            return auth_value == expected_authorization;
        }

        false
    } else {
        false
    }
}

//     let authorization = find_header_value(&request, "Authorization");

//     if let Some(authorization) = authorization {
//         debug!("Authorization header found: {}", authorization);
//     } else {
//         let response = "HTTP/1.1 401 Unauthorized\r\n\
//          WWW-Authenticate: Basic realm=\"Production\"\r\n\
//          Content-Length: 0\r\n\
//          \r\n";
//         stream.write_all(response.as_bytes()).await.unwrap();
//         stream.shutdown().await.unwrap();
//         debug!("No authorization header found.");
//     }

//     ResolvedClient {
//         is_authorized: authorization,
//         initial_request: request,
//         resolved_host: hostname,
//     }
// }

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

fn find_header_value(request: &String, header_name: &str) -> Option<String> {
    let header_key_lowercase = format!("{}:", header_name).to_lowercase();

    request
        .lines()
        .find(|line| {
            line.to_lowercase()
                .starts_with(header_key_lowercase.as_str())
        })
        .map(|host_header| {
            let lowercase_header = host_header.to_lowercase();

            let hostname = lowercase_header
                .trim_start_matches(header_key_lowercase.as_str())
                .trim();

            hostname
                .split_once(':')
                .map_or_else(|| hostname.to_string(), |(host, _)| host.to_string())
        })
}
