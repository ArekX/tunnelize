use base64::{engine::general_purpose, Engine as _};

pub fn find_request_host(request: &String) -> Option<String> {
    let host = find_header_value(request, "Host");

    if let Some(host) = host {
        if let Some((host, _)) = host.split_once(':') {
            return Some(host.to_string());
        }

        return Some(host);
    }

    None
}

pub fn is_authorized(request: &String, username: &String, password: &String) -> bool {
    if let Some(authorization) = find_header_value(&request, "Authorization") {
        let expected_authorization =
            general_purpose::STANDARD.encode(format!("{}:{}", username, password));

        if let Some(auth_value) = authorization.split_whitespace().last() {
            return auth_value == expected_authorization;
        }
    }

    false
}

pub fn get_http_version(request: &String) -> String {
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

pub fn get_unauthorized_response(request: &String, realm: &Option<String>) -> String {
    let realm_string = match realm.as_ref() {
        Some(realm) => realm,
        None => "Production",
    };

    let message =
        "Access to the requested resource is not authorized. Please provide valid credentials.";

    format!(
        "{} 401 Unauthorized\r\nWWW-Authenticate: Basic realm=\"{}\"\r\nContent-Length: {}\r\n\r\n{}",
        get_http_version(request),
        realm_string,
        message.len(),
        message
    )
}

pub fn get_error_response(request: &String, message: String) -> String {
    format!(
        "{} 502 OK\r\nContent-Length: {}\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n{}",
        get_http_version(request),
        message.len(),
        message
    )
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
            let header_value = host_header.split_once(':');

            if let Some((_, header_value)) = header_value {
                return header_value.trim().to_string();
            }

            "".to_string()
        })
}
