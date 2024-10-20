use configuration::{EndpointConfiguration, ServerConfiguration};
use endpoints::http::HttpEndpointConfig;
use log::{debug, info};
use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::sync::Arc;
use tokio::io::Result;

use services::Services;
use tokio_util::sync::CancellationToken;

use crate::common::tasks::start_cancel_listener;

mod configuration;
pub mod endpoints;
mod hub_server;
pub mod incoming_requests;
mod services;
mod session;

pub async fn start() -> Result<()> {
    let mut configuration = ServerConfiguration {
        server_port: 3456,
        admin_key: None,
        max_tunnel_input_wait: 30,
        endpoint_key: None,
        endpoints: HashMap::new(),
    }; // TODO: This should be a parameter in start

    configuration.endpoints.insert(
        "http".to_owned(),
        EndpointConfiguration::Http(HttpEndpointConfig {
            port: 3457,
            is_secure: false,
            address: None,
            max_client_input_wait_secs: 10,
            hostname_template: "opop-{name}.localhost".to_owned(),
            full_url_template: None,
            allow_custom_hostnames: true,
            require_authorization: None,
        }),
    );

    let services = Arc::new(Services::new(configuration));

    let cancel_token = CancellationToken::new();

    let server_future = {
        let services = services.clone();
        let cancel_token = cancel_token.clone();
        tokio::spawn(async move {
            if let Err(e) = hub_server::start(services, cancel_token.clone()).await {
                debug!("Error starting hub server: {:?}", e);
            }

            cancel_token.cancel();
        })
    };

    let cancel_future = tokio::spawn(async move { start_cancel_listener(cancel_token).await });

    match tokio::try_join!(server_future, cancel_future) {
        Ok(_) => {
            info!("Server stopped.");
            Ok(())
        }
        Err(_) => Err(Error::new(
            ErrorKind::Other,
            "Error occurred in server run.",
        )),
    }
}
