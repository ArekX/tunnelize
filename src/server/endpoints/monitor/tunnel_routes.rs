use log::error;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get},
    Router,
};
use uuid::Uuid;

use crate::server::{endpoints::monitor::response::into_message, monitoring};

use super::{
    response::{into_json, into_not_found, into_records},
    state::AppState,
};

async fn list_tunnels(State(state): State<AppState>) -> impl IntoResponse {
    into_records(monitoring::get_tunnel_list(&state.services).await)
}

async fn get_tunnel(
    Path(tunnel_id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match monitoring::get_tunnel_info(&state.services, &tunnel_id).await {
        Some(info) => into_json(StatusCode::OK, info),
        None => into_not_found(),
    }
}

async fn disconnect_tunnel(
    Path(tunnel_id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if let Err(error) = monitoring::disconnect_tunnel(&state.services, &tunnel_id).await {
        error!("Failed to cancel tunnel session: {}", error);
        return into_message(StatusCode::NOT_FOUND, &error);
    }

    into_message(StatusCode::OK, "Tunnel disconnected")
}

pub fn get_router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_tunnels))
        .route("/{id}", get(get_tunnel))
        .route("/{id}", delete(disconnect_tunnel))
}
