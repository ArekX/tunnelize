use log::error;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get},
    Router,
};
use uuid::Uuid;

use crate::server::endpoints::monitor::response::into_message;

use super::{
    response::{into_json, into_not_found, into_records},
    state::AppState,
};

async fn list_tunnels(State(state): State<AppState>) -> impl IntoResponse {
    into_records(state.services.get_tunnel_manager().await.list_all_tunnels())
}

async fn get_tunnel(
    Path(tunnel_id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let tunnel_info = state
        .services
        .get_tunnel_manager()
        .await
        .get_tunnel_info(&tunnel_id);

    match tunnel_info {
        Some(info) => into_json(StatusCode::OK, info),
        None => into_not_found(),
    }
}

async fn disconnect_tunnel(
    Path(tunnel_id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if let Err(error) = state
        .services
        .get_tunnel_manager()
        .await
        .cancel_session(&tunnel_id)
    {
        error!("Failed to cancel tunnel session: {}", error);
        return into_message(StatusCode::NOT_FOUND, &error);
    }

    into_message(StatusCode::OK, "Tunnel disconnected")
}

pub fn get_router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_tunnels))
        .route("/:id", get(get_tunnel))
        .route("/:id", delete(disconnect_tunnel))
}
