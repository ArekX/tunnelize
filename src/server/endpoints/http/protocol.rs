use std::collections::HashMap;

use base64::{engine::general_purpose, Engine as _};

use crate::common::connection::ConnectionStream;

pub struct HttpRequestReader {
    request: String,
}

impl HttpRequestReader {
    pub async fn new(stream: &mut ConnectionStream) -> Self {
        Self {
            request: stream.read_string_until("\r\n\r\n").await,
        }
    }

    pub fn find_hostname(&self) -> Option<String> {
        let host = self.find_header_value("Host");

        if let Some(host) = host {
            if let Some((host, _)) = host.split_once(':') {
                return Some(host.to_string());
            }

            return Some(host);
        }

        None
    }

    pub fn is_authorization_matching(&self, username: &str, password: &str) -> bool {
        if let Some(authorization) = self.find_header_value("Authorization") {
            let expected_authorization =
                general_purpose::STANDARD.encode(format!("{}:{}", username, password));

            if let Some(auth_value) = authorization.split_whitespace().last() {
                return auth_value == expected_authorization;
            }
        }

        false
    }

    fn find_header_value(&self, header_name: &str) -> Option<String> {
        let header_key_lowercase = format!("{}:", header_name).to_lowercase();

        self.request
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

    pub fn get_request_bytes(&self) -> Vec<u8> {
        self.request.clone().into_bytes()
    }
}

pub enum HttpStatusCode {
    Unauthorized,
    BadGateway,
}

impl HttpStatusCode {
    pub fn get_status_text(&self) -> &'static str {
        match self {
            HttpStatusCode::Unauthorized => "401 Unauthorized",
            HttpStatusCode::BadGateway => "502 Bad Gateway",
        }
    }
}

pub struct HttpResponseBuilder {
    status_code: HttpStatusCode,
    headers: HashMap<String, String>,
    body: String,
}

impl HttpResponseBuilder {
    pub fn new(status_code: HttpStatusCode, body: &str) -> Self {
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

    pub fn from_unauthorized(realm: &Option<String>, message: &str) -> Self {
        let realm_string = match realm.as_ref() {
            Some(realm) => realm,
            None => "Production",
        };

        let mut instance = Self::new(HttpStatusCode::Unauthorized, message);

        instance.with_header(
            "WWW-Authenticate".to_string(),
            format!("Basic realm=\"{}\"", realm_string.replace('"', "")),
        );

        instance
    }

    pub fn from_error(message: &str) -> Self {
        Self::new(HttpStatusCode::BadGateway, message)
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
            self.status_code.get_status_text(),
            header_string,
            self.body
        )
    }

    pub fn build_bytes(&self) -> Vec<u8> {
        self.build().into_bytes()
    }
}
