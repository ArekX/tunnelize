use std::io::{Error, ErrorKind};
use std::{collections::HashMap, time::Duration};

use base64::{engine::general_purpose, Engine as _};
use tokio::io::Result;
use tokio::time::timeout;

use crate::common::connection::Connection;

pub struct HttpRequestReader {
    request: String,
}

impl HttpRequestReader {
    pub async fn new(stream: &mut Connection, max_input_wait: u64) -> Result<Self> {
        let request = match timeout(
            Duration::from_secs(max_input_wait),
            stream.read_string_until("\r\n\r\n"),
        )
        .await
        {
            Ok(request) => request,
            Err(e) => {
                stream
                    .close_with_data(
                        &HttpResponseBuilder::as_error(
                            "Failed to read request data within allowed time frame",
                        )
                        .build_bytes(),
                    )
                    .await;
                return Err(Error::new(ErrorKind::Other, e));
            }
        };

        Ok(Self::new_from_string(request))
    }

    pub fn new_from_string(request: String) -> Self {
        Self { request }
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

#[derive(PartialEq, Debug)]
pub enum HttpStatusCode {
    Unauthorized,
    MovedPermanently,
    BadRequest,
    BadGateway,
}

impl HttpStatusCode {
    pub fn get_status_text(&self) -> &'static str {
        match self {
            HttpStatusCode::Unauthorized => "401 Unauthorized",
            HttpStatusCode::BadRequest => "400 Bad Request",
            HttpStatusCode::BadGateway => "502 Bad Gateway",
            HttpStatusCode::MovedPermanently => "301 Moved Permanently",
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

    pub fn as_unauthorized(realm: &Option<String>, message: &str) -> Self {
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

    pub fn as_redirect(location: &str) -> Self {
        let mut instance = Self::new(HttpStatusCode::MovedPermanently, "");

        instance
            .with_header("Location".to_string(), location.to_owned())
            .with_header("Connection".to_string(), "close".to_string());

        instance
    }

    pub fn as_error(message: &str) -> Self {
        Self::new(HttpStatusCode::BadGateway, message)
    }

    pub fn as_bad_request(message: &str) -> Self {
        let mut instance = Self::new(HttpStatusCode::BadRequest, message);

        instance.with_header("Connection".to_string(), "close".to_string());

        instance
    }

    pub fn as_missing_header() -> Self {
        Self::as_bad_request("Host header is missing")
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_http_request_reader(request: &str) -> HttpRequestReader {
        HttpRequestReader::new_from_string(request.to_string())
    }

    fn create_http_response_builder(
        status_code: HttpStatusCode,
        body: &str,
    ) -> HttpResponseBuilder {
        HttpResponseBuilder::new(status_code, body)
    }

    #[tokio::test]
    async fn test_http_request_reader_new() {
        let request = "GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
        let reader = create_http_request_reader(request);
        assert_eq!(reader.request, request);
    }

    #[test]
    fn test_http_request_reader_find_hostname() {
        let request = "GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
        let reader = create_http_request_reader(request);
        assert_eq!(reader.find_hostname(), Some("example.com".to_string()));
    }

    #[test]
    fn test_http_request_reader_is_authorization_matching() {
        let request = "GET / HTTP/1.1\r\nAuthorization: Basic dXNlcjpwYXNz\r\n\r\n";
        let reader = create_http_request_reader(request);
        assert!(reader.is_authorization_matching("user", "pass"));
    }

    #[test]
    fn test_http_request_reader_get_request_bytes() {
        let request = "GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
        let reader = create_http_request_reader(request);
        assert_eq!(reader.get_request_bytes(), request.as_bytes().to_vec());
    }

    #[test]
    fn test_http_response_builder_new() {
        let response = create_http_response_builder(HttpStatusCode::BadRequest, "Bad Request");
        assert_eq!(response.status_code, HttpStatusCode::BadRequest);
        assert_eq!(response.body, "Bad Request");
        assert_eq!(response.headers.get("Content-Type").unwrap(), "text/plain");
    }

    #[test]
    fn test_http_response_builder_as_unauthorized() {
        let response =
            HttpResponseBuilder::as_unauthorized(&Some("TestRealm".to_string()), "Unauthorized");
        assert_eq!(response.status_code, HttpStatusCode::Unauthorized);
        assert_eq!(
            response.headers.get("WWW-Authenticate").unwrap(),
            "Basic realm=\"TestRealm\""
        );
    }

    #[test]
    fn test_http_response_builder_as_redirect() {
        let response = HttpResponseBuilder::as_redirect("http://example.com");
        assert_eq!(response.status_code, HttpStatusCode::MovedPermanently);
        assert_eq!(
            response.headers.get("Location").unwrap(),
            "http://example.com"
        );
    }

    #[test]
    fn test_http_response_builder_as_error() {
        let response = HttpResponseBuilder::as_error("Error occurred");
        assert_eq!(response.status_code, HttpStatusCode::BadGateway);
        assert_eq!(response.body, "Error occurred");
    }

    #[test]
    fn test_http_response_builder_as_bad_request() {
        let response = HttpResponseBuilder::as_bad_request("Bad Request");
        assert_eq!(response.status_code, HttpStatusCode::BadRequest);
        assert_eq!(response.body, "Bad Request");
    }

    #[test]
    fn test_http_response_builder_as_missing_header() {
        let response = HttpResponseBuilder::as_missing_header();
        assert_eq!(response.status_code, HttpStatusCode::BadRequest);
        assert_eq!(response.body, "Host header is missing");
    }

    #[test]
    fn test_http_response_builder_build() {
        let response = create_http_response_builder(HttpStatusCode::BadRequest, "Bad Request");
        let built_response = response.build();
        assert!(built_response.contains("HTTP/1.1 400 Bad Request"));
        assert!(built_response.contains("Content-Length: 11"));
    }

    #[test]
    fn test_http_response_builder_build_bytes() {
        let response = create_http_response_builder(HttpStatusCode::BadRequest, "Bad Request");
        let built_response_bytes = response.build_bytes();
        assert_eq!(built_response_bytes, response.build().into_bytes());
    }
}
