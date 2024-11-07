use std::{collections::HashMap, fs::File, io::BufWriter};

use log::info;

use crate::{
    common::{cli::InitCommands, tcp_server::ServerEncryption},
    server::{
        configuration::{
            self, EndpointConfiguration, EndpointServerEncryption, ServerConfiguration,
        },
        endpoints::{
            http::configuration::HttpEndpointConfig,
            monitor::configuration::{MonitorAuthentication, MonitorEndpointConfig, MonitorOrigin},
            tcp::configuration::TcpEndpointConfig,
            udp::configuration::UdpEndpointConfig,
        },
    },
};

pub fn get_default_server_configuration() -> ServerConfiguration {
    let mut configuration = ServerConfiguration {
        server_port: 3456,
        server_address: None,
        monitor_key: Some("changethiskey".to_owned()),
        max_tunnel_input_wait: 30,
        tunnel_key: Some("changethistunnelkey".to_owned()),
        endpoints: HashMap::new(),
        max_tunnels: 50,
        max_clients: 100,
        encryption: ServerEncryption::None,
    };

    configuration.endpoints.insert(
        "http".to_owned(),
        EndpointConfiguration::Http(HttpEndpointConfig {
            port: 3457,
            encryption: EndpointServerEncryption::None,
            address: None,
            max_client_input_wait_secs: 10,
            hostname_template: "tunnel-{name}.localhost".to_owned(),
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
            allow_cors_origins: MonitorOrigin::Any,
            authentication: MonitorAuthentication::Basic {
                username: "admin".to_owned(),
                password: "changethispassword".to_owned(),
            },
        }),
    );

    configuration.endpoints.insert(
        "tcp".to_owned(),
        EndpointConfiguration::Tcp(TcpEndpointConfig {
            reserve_ports_from: 4000,
            reserve_ports_to: 4002,
            allow_desired_port: true,
            encryption: EndpointServerEncryption::None,
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
            inactivity_timeout: 20,
            full_hostname_template: Some("127.0.0.1:{port}".to_owned()),
            address: None,
        }),
    );

    configuration
}

pub async fn init_for(command: InitCommands) -> Result<(), std::io::Error> {
    match command {
        InitCommands::Server => {
            let configuration = get_default_server_configuration();

            let file = BufWriter::new(File::create("./tunnelize.json")?);

            serde_json::to_writer_pretty(file, &configuration)?;

            info!("Initializing server...");
        }
        InitCommands::Tunnel { server, cert, key } => {
            info!("Initializing tunnel...");
        }
    }

    Ok(())
}
