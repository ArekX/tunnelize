use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use log::debug;
use serde::Serialize;

use super::state::AppState;

#[derive(Debug, Serialize)]
struct ErrorResponse {
    message: String,
}

pub async fn handle_default_response(
    request: Request,
    next: Next,
) -> std::result::Result<impl IntoResponse, Response> {
    let response = next.run(request).await;

    if response.headers().get("content-type") == Some(&"application/json".parse().unwrap()) {
        return Ok((response.status(), response));
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

    Ok((status_code, Json(ErrorResponse { message }).into_response()))
}

pub async fn handle_authorization(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> std::result::Result<impl IntoResponse, Response> {
    debug!(
        "Checking authorization {:?}",
        state.services.get_config().admin_key
    );
    Ok(next.run(request).await)
}
