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

#[cfg(test)]
mod tests {
    use std::usize;

    use super::*;
    use axum::body::Body;
    use axum::http::StatusCode;
    use axum::response::Response;
    use serde_json::Value;

    async fn extract_body(response: Response<Body>) -> Value {
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        serde_json::from_slice(&body).unwrap()
    }

    #[tokio::test]
    async fn test_into_json() {
        let data = json!({ "key": "value" });
        let response = into_json(StatusCode::OK, data.clone());
        assert_eq!(response.status(), StatusCode::OK);

        let body = extract_body(response).await;
        assert_eq!(body, data);
    }

    #[tokio::test]
    async fn test_into_records() {
        #[derive(Serialize, Clone)]
        struct Record {
            id: u32,
        }

        let records = vec![Record { id: 1 }, Record { id: 2 }];
        let response = into_records(records.clone());
        assert_eq!(response.status(), StatusCode::OK);

        let body = extract_body(response).await;
        let expected = json!({ "records": records });
        assert_eq!(body, expected);
    }

    #[tokio::test]
    async fn test_into_message() {
        let message = "Test message";
        let response = into_message(StatusCode::BAD_REQUEST, message);
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = extract_body(response).await;
        let expected = json!({ "message": message });
        assert_eq!(body, expected);
    }

    #[tokio::test]
    async fn test_into_not_found() {
        let response = into_not_found();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = extract_body(response).await;
        let expected = json!({ "message": "Requested item not found" });
        assert_eq!(body, expected);
    }
}
