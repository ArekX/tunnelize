use std::collections::HashMap;

use base64::{engine::general_purpose, Engine as _};
use log::debug;

use crate::common::connection::ConnectionStream;

enum HttpStatusCode {
    Unauthorized,
    BadGateway,
}

impl HttpStatusCode {
    pub fn get_protocol_text(&self) -> &'static str {
        match self {
            HttpStatusCode::Unauthorized => "401 Unauthorized",
            HttpStatusCode::BadGateway => "502 Bad Gateway",
        }
    }
}

struct HttpResponseBuilder {
    status_code: HttpStatusCode,
    headers: HashMap<String, String>,
    body: String,
}

impl HttpResponseBuilder {
    pub fn from_request(status_code: HttpStatusCode, body: &str) -> Self {
        let mut instance = Self {
            status_code,
            body: body.to_owned(),
            headers: HashMap::new(),
        };

        instance
            .with_header("Content-Type".to_string(), "text/plain".to_string())
            .with_header("Content-Length".to_string(), body.len().to_string())
            .with_header("Connection".to_string(), "close".to_string());

        instance
    }

    pub fn with_header(&mut self, header: String, value: String) -> &mut Self {
        self.headers.insert(header, value);
        self
    }

    pub fn build(&self) -> String {
        let header_string = self
            .headers
            .iter()
            .map(|(header, value)| format!("{}: {}", header, value))
            .collect::<Vec<String>>()
            .join("\r\n");

        format!(
            "HTTP/1.1 {}\r\n{}\r\n\r\n{}",
            self.status_code.get_protocol_text(),
            header_string,
            self.body
        )
    }
}

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

pub fn get_unauthorized_response(realm: &Option<String>) -> String {
    let realm_string = match realm.as_ref() {
        Some(realm) => realm,
        None => "Production",
    };

    HttpResponseBuilder::from_request(
        HttpStatusCode::Unauthorized,
        "Access to the requested resource is not authorized. Please provide valid credentials.",
    )
    .with_header(
        "WWW-Authenticate".to_string(),
        format!("Basic realm=\"{}\"", realm_string),
    )
    .build()
}

pub fn get_error_response(message: &str) -> String {
    HttpResponseBuilder::from_request(HttpStatusCode::BadGateway, message).build()
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
