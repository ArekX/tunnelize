use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use uuid::Uuid;

use crate::server::monitoring::{self};

use super::{
    response::{into_json, into_not_found, into_records},
    state::AppState,
};

async fn get_system_info(State(state): State<AppState>) -> impl IntoResponse {
    into_json(
        StatusCode::OK,
        monitoring::get_system_info(&state.services).await,
    )
}

async fn list_endpoints(State(state): State<AppState>) -> impl IntoResponse {
    into_records(monitoring::get_endpoint_list(&state.services).await)
}

async fn get_endpoint(
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match monitoring::get_endpoint_info(&state.services, &name).await {
        Some(info) => into_json(StatusCode::OK, info),
        None => into_not_found(),
    }
}

async fn list_clients(State(state): State<AppState>) -> impl IntoResponse {
    into_records(monitoring::get_client_list(&state.services).await)
}

async fn get_client(
    Path(client_id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match monitoring::get_client_info(&state.services, &client_id).await {
        Some(info) => into_json(StatusCode::OK, info),
        None => into_not_found(),
    }
}

pub fn get_router() -> Router<AppState> {
    Router::new()
        .route("/endpoints", get(list_endpoints))
        .route("/endpoints/{name}", get(get_endpoint))
        .route("/clients", get(list_clients))
        .route("/clients/{id}", get(get_client))
        .route("/info", get(get_system_info))
}
