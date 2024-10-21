use serde::Serialize;
use uuid::Uuid;

use axum::{extract::State, http::StatusCode, routing::get, Json, Router};

use super::state::AppState;

#[derive(Serialize)]
struct TunnelList {
    tunnels: Vec<Uuid>,
}

async fn list_tunnels(State(state): State<AppState>) -> (StatusCode, Json<TunnelList>) {
    (
        StatusCode::OK,
        Json(TunnelList {
            tunnels: state.services.get_tunnel_manager().await.get_tunnel_ids(),
        }),
    )
}

pub fn get_tunnel_routes() -> Router<AppState> {
    Router::new().route("/", get(list_tunnels))
}
