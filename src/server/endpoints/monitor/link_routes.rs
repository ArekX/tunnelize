use crate::server::{endpoints::monitor::response::into_message, monitoring};
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

async fn list_links(State(state): State<AppState>) -> impl IntoResponse {
    into_records(monitoring::get_link_list(&state.services).await)
}

async fn get_link(
    Path(session_id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match monitoring::get_link_info(&state.services, &session_id).await {
        Some(info) => into_json(StatusCode::OK, info),
        None => into_not_found(),
    }
}

async fn disconnect_link(
    Path(session_id): Path<Uuid>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if let Err(error) = monitoring::disconnect_link(&state.services, &session_id).await {
        error!("Failed to cancel link session: {}", error);
        return into_message(StatusCode::NOT_FOUND, &error);
    }

    into_message(StatusCode::OK, "Link disconnected")
}

pub fn get_router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_links))
        .route("/:id", get(get_link))
        .route("/:id", delete(disconnect_link))
}
