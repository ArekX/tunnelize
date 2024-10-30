use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use axum_server::tls_rustls::RustlsConfig;
use configuration::MonitorEndpointConfig;
use log::error;
use state::AppState;
use tokio::io::Result;

use crate::{
    common::{channel::RequestReceiver, tcp_server::ServerEncryption},
    server::{configuration::EndpointServerEncryption, services::Services},
};

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

    let state = AppState::new(services.clone(), config.clone(), name.clone());

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

    // TODO: Add cancellation token
    tokio::spawn(async move {
        loop {
            match channel_rx.wait_for_requests().await {
                Some(request) => {
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

    let Ok(address) = config.get_bind_address().parse::<SocketAddr>() else {
        return Err(tokio::io::Error::new(
            tokio::io::ErrorKind::AddrNotAvailable,
            "Failed to parse address.",
        ));
    };

    match config.encryption {
        EndpointServerEncryption::None => start_http_server(address, app).await,
        EndpointServerEncryption::CustomTls {
            ref cert_path,
            ref key_path,
        } => start_https_server(address, cert_path, key_path, app).await,
        EndpointServerEncryption::ServerTls => {
            let main_config = services.get_config();

            let (cert_path, key_path) = match main_config.encryption {
                ServerEncryption::Tls {
                    ref cert_path,
                    ref key_path,
                } => (cert_path, key_path),
                ServerEncryption::None => {
                    return Err(tokio::io::Error::new(
                        tokio::io::ErrorKind::InvalidInput,
                        format!("Tunnel server TLS encryption is not set, but required by monitor '{}' endpoint", name),
                    ));
                }
            };

            start_https_server(address, cert_path, key_path, app).await
        }
    }
}

async fn start_http_server(address: SocketAddr, app: Router) -> Result<()> {
    axum_server::bind(address)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
}

async fn start_https_server(
    address: SocketAddr,
    cert_path: &str,
    key_path: &str,
    app: Router,
) -> Result<()> {
    let config =
        RustlsConfig::from_pem_file(PathBuf::from(cert_path), PathBuf::from(key_path)).await?;

    axum_server::bind_rustls(address, config)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
}
