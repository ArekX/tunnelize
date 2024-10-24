use std::{net::SocketAddr, sync::Arc};

use configuration::MonitorEndpointConfig;
use log::{debug, error};
use state::AppState;
use tokio::{io::Result, net::TcpListener};

use crate::{common::channel::RequestReceiver, server::services::Services};

use super::messages::EndpointChannelRequest;
use axum::{
    middleware::{from_fn, from_fn_with_state},
    Router,
};

mod channel_handler;
pub mod configuration;
mod link_routes;
mod middleware;
mod response;
mod state;
mod system_routes;
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
        .layer(from_fn_with_state(
            state.clone(),
            middleware::handle_authorization,
        ))
        .nest("/tunnels", tunnel_routes::get_router())
        .nest("/links", link_routes::get_router())
        .nest("/system", system_routes::get_router())
        .layer(from_fn(middleware::handle_default_response))
        .with_state(state.clone());

    let listener = TcpListener::bind(config.get_bind_address()).await?;

    tokio::spawn(async move {
        loop {
            match channel_rx.wait_for_requests().await {
                Some(request) => {
                    debug!("Received endpoint message");
                    if let Err(e) = channel_handler::handle(request).await {
                        error!("Failed to handle endpoint message: {}", e);
                    }
                }
                None => {
                    break;
                }
            }
        }
    });

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}
