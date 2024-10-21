use std::{net::SocketAddr, sync::Arc};

use configuration::MonitorEndpointConfig;
use state::AppState;
use tokio::{io::Result, net::TcpListener};
use tunnel_routes::get_tunnel_routes;

use crate::{common::channel::RequestReceiver, server::services::Services};

use super::messages::EndpointChannelRequest;
use axum::{
    middleware::{from_fn, from_fn_with_state},
    Router,
};

pub mod configuration;
mod middleware;
mod state;
mod tunnel_routes;

pub async fn start(
    services: Arc<Services>,
    name: String,
    config: MonitorEndpointConfig,
    mut channel_rx: RequestReceiver<EndpointChannelRequest>,
) -> Result<()> {
    // TODO: Add CORS
    let config = Arc::new(config);

    let state = AppState::new(services.clone(), config.clone(), name);

    let app = Router::new()
        .layer(from_fn(middleware::handle_default_response))
        .layer(from_fn_with_state(
            state.clone(),
            middleware::handle_authorization,
        ))
        .nest("/tunnel", get_tunnel_routes())
        .with_state(state.clone());

    let listener = TcpListener::bind(config.get_bind_address()).await?;

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
