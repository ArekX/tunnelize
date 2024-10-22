use std::{fmt::Result, net::SocketAddr, os::linux::raw::stat};

use axum::{
    body::Body,
    extract::{ConnectInfo, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use base64::{engine::general_purpose, Engine as _};
use log::debug;
use serde::Serialize;

use crate::server::endpoints::monitor::configuration::MonitorAuthentication;

use super::state::AppState;

#[derive(Debug, Serialize)]
struct ErrorResponse {
    message: String,
}

fn to_error_response(status_code: StatusCode, message: &str) -> Response<Body> {
    (
        status_code,
        Json(ErrorResponse {
            message: message.to_owned(),
        }),
    )
        .into_response()
}

pub async fn handle_default_response(
    request: Request,
    next: Next,
) -> std::result::Result<impl IntoResponse, Response> {
    let response = next.run(request).await;

    if response.headers().get("content-type") == Some(&"application/json".parse().unwrap()) {
        return Ok((response.status(), response).into_response());
    }

    let status_code = response.status();
    let message = match status_code {
        StatusCode::NOT_FOUND => "Requested resource not found".to_owned(),
        StatusCode::METHOD_NOT_ALLOWED => "Method not allowed.".to_owned(),
        StatusCode::BAD_REQUEST => "Bad request".to_owned(),
        StatusCode::UNAUTHORIZED => "You are not authorized to access this endpoint.".to_owned(),
        StatusCode::INTERNAL_SERVER_ERROR => {
            match axum::body::to_bytes(response.into_body(), 2048usize).await {
                Ok(body_bytes) => match String::from_utf8(body_bytes.to_vec()) {
                    Ok(string) => string,
                    Err(e) => {
                        debug!("Error converting response body to string: {}", e);
                        "Unknown error".to_owned()
                    }
                },
                Err(e) => {
                    debug!("Error reading response body: {}", e);
                    "Unknown error".to_owned()
                }
            }
        }
        _ => "Unknown error".to_owned(),
    };

    Ok(to_error_response(status_code, message.as_str()))
}

pub async fn handle_authorization(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> std::result::Result<impl IntoResponse, Response> {
    let Some(authorization) = request.headers().get("Authorization") else {
        if let MonitorAuthentication::None = state.config.authentication {
            return Ok(next.run(request).await);
        }

        return Ok(to_auth_error_response(
            &state.config.authentication,
            &state.name,
            "Authorization header not found",
        ));
    };

    if state.is_locked(&addr.ip()).await {
        return Ok(to_error_response(
            StatusCode::TOO_MANY_REQUESTS,
            "Too many failed attempts. Please try again later.",
        ));
    }

    let Some(auth_value) = authorization
        .to_str()
        .ok()
        .and_then(|auth| auth.split_whitespace().last().map(|auth| auth.to_string()))
    else {
        return Ok(to_auth_error_response(
            &state.config.authentication,
            &state.name,
            "Invalid authorization header",
        ));
    };

    Ok(
        match check_authentication(&auth_value, &state.config.authentication) {
            Ok(_) => {
                state.clear_ip_attempts(&addr.ip()).await;
                next.run(request).await
            }
            Err(e) => {
                state.log_ip_attempt(&addr.ip()).await;
                to_auth_error_response(&state.config.authentication, &state.name, e.as_str())
                    .into_response()
            }
        },
    )
}

fn to_auth_error_response(
    auth: &MonitorAuthentication,
    name: &str,
    message: &str,
) -> Response<Body> {
    let mut response = to_error_response(StatusCode::UNAUTHORIZED, message).into_response();

    if let MonitorAuthentication::Basic { .. } = auth {
        response.headers_mut().insert(
            "WWW-Authenticate",
            format!("Basic realm=\"{}\"", name).parse().unwrap(),
        );
    }

    response
}

fn check_authentication(
    auth_value: &str,
    authentication: &MonitorAuthentication,
) -> std::result::Result<(), String> {
    match authentication {
        MonitorAuthentication::None => Ok(()),
        MonitorAuthentication::Basic { username, password } => {
            let expected_authorization =
                general_purpose::STANDARD.encode(format!("{}:{}", username, password));

            if auth_value == expected_authorization {
                return Ok(());
            }

            return Err("Invalid authorization header".to_owned());
        }
        MonitorAuthentication::Bearer { token } => {
            if auth_value.eq(token) {
                return Ok(());
            }

            return Err("Invalid authorization header".to_owned());
        }
    }
}