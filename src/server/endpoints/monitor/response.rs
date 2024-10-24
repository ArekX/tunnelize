use axum::{
    body::Body,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use serde_json::json;

use crate::server::monitoring::Records;

pub fn into_json(status: StatusCode, data: impl Serialize) -> Response<Body> {
    (status, Json(data)).into_response()
}

pub fn into_records<T: Serialize>(records: Vec<T>) -> Response<Body> {
    into_json(StatusCode::OK, Records { records })
}

pub fn into_message(status: StatusCode, message: &str) -> Response<Body> {
    into_json(status, json!({ "message": message }))
}

pub fn into_not_found() -> Response<Body> {
    into_message(StatusCode::NOT_FOUND, "Requested item not found")
}
