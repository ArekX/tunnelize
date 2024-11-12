use std::{net::SocketAddr, path::PathBuf, sync::Arc};

use axum_server::tls_rustls::RustlsConfig;
use configuration::{MonitorEndpointConfig, MonitorOrigin};
use log::{error, info};
use state::AppState;
use tokio::io::Result;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};

use crate::{
    common::{
        channel::{InvalidResponse, OkResponse, RequestReceiver},
        configuration::ServerEncryption,
    },
    server::{configuration::EndpointServerEncryption, services::Services},
};

use super::messages::EndpointChannelRequest;
use axum::{
    http::HeaderValue,
    middleware::{from_fn, from_fn_with_state},
    Router,
};

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
    channel_rx: RequestReceiver<EndpointChannelRequest>,
) -> Result<()> {
    let config = Arc::new(config);

    let state = AppState::new(services.clone(), config.clone(), name.clone());

    let mut app = Router::new()
        .layer(from_fn_with_state(
            state.clone(),
            middleware::handle_authorization,
        ))
        .nest("/tunnels", tunnel_routes::get_router())
        .nest("/links", link_routes::get_router())
        .nest("/system", system_routes::get_router())
        .layer(from_fn(middleware::handle_default_response))
        .with_state(state.clone());

    app = apply_cors(app, &config);

    let Ok(address) = config.get_bind_address().parse::<SocketAddr>() else {
        return Err(tokio::io::Error::new(
            tokio::io::ErrorKind::AddrNotAvailable,
            "Failed to parse address.",
        ));
    };

    let cancel_token = services.get_cancel_token();

    tokio::select! {
        _ = cancel_token.cancelled() => {
            info!("Monitor '{}' server cancelled", name);
        }
        _ = start_channel_handler(channel_rx) => {
            info!("Monitor '{}' channel handler stopped", name);
        }
        result = start_server(address, app, config, name.clone(), services.clone()) => {
            if let Err(e) = result {
                error!("Failed to start monitor server '{}': {}", name, e);
            }
        }
    }

    Ok(())
}

fn start_channel_handler(
    mut channel_rx: RequestReceiver<EndpointChannelRequest>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            match channel_rx.wait_for_requests().await {
                Some(ref mut request) => match &request.data {
                    EndpointChannelRequest::RemoveTunnelRequest(_) => {
                        request.respond(OkResponse);
                    }
                    _ => {
                        request.respond(InvalidResponse);
                    }
                },
                None => {
                    break;
                }
            }
        }
    })
}

fn apply_cors(app: Router, config: &MonitorEndpointConfig) -> Router {
    match config.allow_cors_origins {
        MonitorOrigin::Any => {
            return app.layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any),
            );
        }
        MonitorOrigin::List(ref origins) => {
            let origins = origins
                .iter()
                .filter_map(|o| o.to_lowercase().parse().ok())
                .collect::<Vec<HeaderValue>>();

            let cors = CorsLayer::new()
                .allow_methods(Any)
                .allow_headers(Any)
                .allow_origin(AllowOrigin::predicate(move |origin, _headers| {
                    origins.iter().any(|allowed_origin| {
                        return origin == allowed_origin;
                    })
                }));

            return app.layer(cors);
        }
        MonitorOrigin::None => {
            return app;
        }
    }
}

async fn start_server(
    address: SocketAddr,
    app: Router,
    config: Arc<MonitorEndpointConfig>,
    name: String,
    services: Arc<Services>,
) -> Result<()> {
    match config.encryption {
        EndpointServerEncryption::None => start_http_server(address, app).await,
        EndpointServerEncryption::CustomTls {
            ref cert_path,
            ref key_path,
        } => start_https_server(address, cert_path, key_path, app).await,
        EndpointServerEncryption::Tls => {
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
