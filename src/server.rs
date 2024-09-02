use axum::Extension;
use axum::{routing::get, Router};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Config {
    route: Option<String>,
    domain: Option<String>,
    client_ip: String,
}

pub async fn start_server() {
    let config: Arc<RwLock<Config>> = Arc::new(RwLock::new(Config {
        route: None,
        domain: None,
        client_ip: "".to_string(),
    }));

    let app = Router::new()
        .route("/config", get(get_config_handler))
        .route("/r/{route_or_domain}", get(route_handler))
        .layer(Extension(config.clone()));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn get_config_handler() -> String {
    // Return the current configuration as JSON
    // You can modify this function to return the stored configuration
    // from the `config` variable
    let config = Config {
        route: Some("/example".to_owned()),
        domain: None,
        client_ip: "127.0.0.1".to_owned(),
    };

    serde_json::to_string(&config).unwrap()
}

async fn route_handler(
    axum::extract::Path(route_or_domain): axum::extract::Path<String>,
    axum::extract::Extension(config): axum::extract::Extension<
        Arc<RwLock<HashMap<String, Config>>>,
    >,
) -> String {
    // Retrieve the configuration based on the provided route or domain
    let config = config.read().unwrap();
    let config = config.get(&route_or_domain).cloned();

    match config {
        Some(config) => {
            // Forward the request to the client's IP based on the configuration
            format!("Forwarding request to client IP: {}", config.client_ip)
        }
        None => "No matching configuration found".to_owned(),
    }
}
