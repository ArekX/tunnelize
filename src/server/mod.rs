use configuration::{EndpointConfiguration, EndpointServerEncryption, ServerConfiguration};
use endpoints::http::configuration::HttpEndpointConfig;
use endpoints::monitor::configuration::{MonitorAuthentication, MonitorEndpointConfig};
use endpoints::tcp::configuration::TcpEndpointConfig;
use endpoints::udp::configuration::UdpEndpointConfig;
use log::{debug, info};
use std::collections::HashMap;
use std::io::{Error, ErrorKind};
use std::sync::Arc;
use tokio::io::Result;

use services::Services;
use tokio_util::sync::CancellationToken;

use crate::common::tasks::start_cancel_listener;
use crate::common::tcp_server::ServerEncryption;

mod configuration;
pub mod endpoints;
mod hub_server;
pub mod incoming_requests;
mod monitoring;
mod services;
mod session;

pub async fn start() -> Result<()> {
    let mut configuration = ServerConfiguration {
        server_port: 3456,
        server_address: None,
        monitor_key: Some("key".to_owned()),
        max_tunnel_input_wait: 30,
        tunnel_key: None,
        endpoints: HashMap::new(),
        encryption: ServerEncryption::Tls {
            cert_path: "testing/certs/server.crt".to_owned(),
            key_path: "testing/certs/server.key".to_owned(),
        },
    }; // TODO: This should be a parameter in start

    configuration.endpoints.insert(
        "http".to_owned(),
        EndpointConfiguration::Http(HttpEndpointConfig {
            port: 3457,
            encryption: EndpointServerEncryption::None,
            address: None,
            max_client_input_wait_secs: 10,
            hostname_template: "opop-{name}.localhost".to_owned(),
            full_url_template: None,
            allow_custom_hostnames: true,
            require_authorization: None,
        }),
    );

    configuration.endpoints.insert(
        "monitor".to_owned(),
        EndpointConfiguration::Monitoring(MonitorEndpointConfig {
            encryption: EndpointServerEncryption::None,
            port: 3000,
            address: None,
            authentication: MonitorAuthentication::Bearer {
                token: "opop".to_owned(),
            },
        }),
    );

    configuration.endpoints.insert(
        "tcp".to_owned(),
        EndpointConfiguration::Tcp(TcpEndpointConfig {
            reserve_ports_from: 4000,
            reserve_ports_to: 4002,
            allow_desired_port: true,
            full_hostname_template: Some("127.0.0.1:{port}".to_owned()),
            address: None,
        }),
    );

    configuration.endpoints.insert(
        "udp".to_owned(),
        EndpointConfiguration::Udp(UdpEndpointConfig {
            reserve_ports_from: 5000,
            allow_desired_port: true,
            reserve_ports_to: 5002,
            inactivity_timeout: 60,
            full_hostname_template: Some("127.0.0.1:{port}".to_owned()),
            address: None,
        }),
    );

    let cancel_token = CancellationToken::new();
    let services = Arc::new(Services::new(configuration, cancel_token.clone()));

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
