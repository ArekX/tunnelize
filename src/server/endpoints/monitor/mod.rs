use std::{os::linux::raw::stat, sync::Arc};

use configuration::MonitorEndpointConfig;
use serde::{Deserialize, Serialize};
use tokio::io::Result;
use uuid::Uuid;

use crate::{common::channel::RequestReceiver, server::services::Services};

use super::messages::EndpointChannelRequest;
use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};

pub mod configuration;

#[derive(Clone)]
struct AppState {
    pub services: Arc<Services>,
}

pub async fn start(
    services: Arc<Services>,
    name: String,
    config: MonitorEndpointConfig,
    mut channel_rx: RequestReceiver<EndpointChannelRequest>,
) -> Result<()> {
    let state = AppState { services };

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(list_tunnels))
        .with_state(state);
    // `POST /users` goes to `create_user`

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(config.get_bind_address())
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

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
