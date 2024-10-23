use crate::server::endpoints::monitor::response::into_message;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get},
    Router,
};
use log::error;
use uuid::Uuid;

use super::{
    response::{into_json, into_not_found, into_records},
    state::AppState,
};

async fn list_clients(State(state): State<AppState>) -> impl IntoResponse {
    into_records(state.services.get_link_manager().await.list_all_sessions())
}

async fn get_link(
    Path(session_id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let tunnel_info = state
        .services
        .get_link_manager()
        .await
        .get_session_info(&session_id);

    match tunnel_info {
        Some(info) => into_json(StatusCode::OK, info),
        None => into_not_found(),
    }
}

async fn disconnect_client(
    Path(session_id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if let Err(error) = state
        .services
        .get_link_manager()
        .await
        .cancel_session(&session_id)
    {
        error!("Failed to cancel tunnel session: {}", error);
        return into_message(StatusCode::NOT_FOUND, &error);
    }

    into_message(StatusCode::OK, "Client disconnected")
}

pub fn get_router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_clients))
        .route("/:id", get(get_link))
        .route("/:id", delete(disconnect_client))
}
