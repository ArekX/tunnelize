use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};

use super::{
    response::{into_json, into_not_found, into_records},
    state::AppState,
};

async fn list_endpoints(State(state): State<AppState>) -> impl IntoResponse {
    into_records(state.services.get_endpoint_manager().await.list_endpoints())
}

async fn get_endpoint(
    Path(name): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let info = state
        .services
        .get_endpoint_manager()
        .await
        .get_endpoint_info(&name);

    match info {
        Some(info) => into_json(StatusCode::OK, info),
        None => into_not_found(),
    }
}

pub fn get_router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_endpoints))
        .route("/:name", get(get_endpoint))
}
