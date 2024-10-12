use base64::{engine::general_purpose, Engine as _};
use log::debug;

use crate::common::connection::ConnectionStream;

pub fn find_host_header_value(http_request: &str) -> Option<String> {
    let host = find_header_value(http_request, "Host");

    if let Some(host) = host {
        if let Some((host, _)) = host.split_once(':') {
            return Some(host.to_string());
        }

        return Some(host);
    }

    None
}

pub fn is_authorized(http_request: &str, username: &str, password: &str) -> bool {
    if let Some(authorization) = find_header_value(&http_request, "Authorization") {
        let expected_authorization =
            general_purpose::STANDARD.encode(format!("{}:{}", username, password));

        if let Some(auth_value) = authorization.split_whitespace().last() {
            return auth_value == expected_authorization;
        }
    }

    false
}

pub fn get_unauthorized_response(http_request: &str, realm: &Option<String>) -> String {
    let realm_string = match realm.as_ref() {
        Some(realm) => realm,
        None => "Production",
    };

    let message =
        "Access to the requested resource is not authorized. Please provide valid credentials.";

    format!(
        "{} 401 Unauthorized\r\nWWW-Authenticate: Basic realm=\"{}\"\r\nContent-Length: {}\r\n\r\n{}",
        get_http_version(http_request),
        realm_string,
        message.len(),
        message
    )
}

pub fn get_error_response(http_request: &str, message: &str) -> String {
    format!(
        "{} 502 OK\r\nContent-Length: {}\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n{}",
        get_http_version(http_request),
        message.len(),
        message
    )
}

pub async fn read_http_request(stream: &mut ConnectionStream) -> String {
    let mut request_buffer = Vec::new();

    loop {
        debug!("Waiting tcp stream to be readable...");

        if let Err(e) = stream.wait_for_messages().await {
            debug!(
                "Error while waiting for client stream to be readable: {:?}",
                e
            );
            break;
        }

        let mut buffer = [0; 100024];

        match stream.read(&mut buffer).await {
            Ok(0) => {
                break;
            }
            Ok(read) => {
                request_buffer.extend_from_slice(&buffer[..read]);

                if String::from_utf8_lossy(&request_buffer).contains("\r\n\r\n") {
                    break;
                }
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

fn find_header_value(request: &str, header_name: &str) -> Option<String> {
    let header_key_lowercase = format!("{}:", header_name).to_lowercase();

    request
        .lines()
        .find(|line| {
            line.to_lowercase()
                .starts_with(header_key_lowercase.as_str())
        })
        .map(|host_header| {
            let header_value = host_header.split_once(':');

            if let Some((_, header_value)) = header_value {
                return header_value.trim().to_string();
            }

            "".to_string()
        })
}

fn get_http_version(request: &str) -> String {
    match request
        .lines()
        .find(|line| line.starts_with("HTTP/"))
        .map(|line| {
            let Some(version) = line.split_whitespace().next() else {
                return "HTTP/1.1";
            };

            version
        }) {
        Some(http_version) => http_version.to_string(),
        None => "HTTP/1.1".to_string(),
    }
}
