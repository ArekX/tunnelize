use serde::Serialize;

use axum::{extract::State, http::StatusCode, routing::get, Json, Router};

use crate::server::services::TunnelInfo;

use super::state::AppState;

#[derive(Serialize)]
struct TunnelList {
    tunnels: Vec<TunnelInfo>,
}

async fn list_tunnels(State(state): State<AppState>) -> (StatusCode, Json<TunnelList>) {
    (
        StatusCode::OK,
        Json(TunnelList {
            tunnels: state.services.get_tunnel_manager().await.get_tunnel_info(),
        }),
    )
}

pub fn get_tunnel_routes() -> Router<AppState> {
    Router::new().route("/", get(list_tunnels))
}
