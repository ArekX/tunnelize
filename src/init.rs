use std::collections::HashMap;

use crate::{
    common::{
        cli::InitCommands, configuration::ServerEncryption, encryption::ClientEncryptionType,
        tcp_client::create_tcp_client,
    },
    configuration::write_configuration,
    server::{
        configuration::{EndpointConfiguration, EndpointServerEncryption, ServerConfiguration},
        endpoints::{
            http::configuration::HttpEndpointConfig,
            monitor::configuration::{MonitorAuthentication, MonitorEndpointConfig, MonitorOrigin},
            tcp::configuration::TcpEndpointConfig,
            udp::configuration::UdpEndpointConfig,
        },
        incoming_requests::{
            ConfigRequest, ProcessConfigRequest, ProcessConfigResponse, PublicEndpointConfig,
            PublicServerEndpointConfig,
        },
    },
    tunnel::configuration::{Encryption, ProxyConfiguration, TunnelConfiguration, TunnelProxy},
};

pub async fn init_for(command: InitCommands) -> Result<(), std::io::Error> {
    match command {
        InitCommands::Server => {
            write_configuration(get_default_server_configuration().into())?;
        }
        InitCommands::Tunnel {
            server,
            cert,
            tls,
            key,
        } => {
            let Some(mut server_address) = server else {
                write_configuration(get_default_tunnel_configuration().into())?;

                return Ok(());
            };

            println!("Connecting to server at {}", server_address);

            if server_address.starts_with("http://") || server_address.starts_with("https://") {
                server_address = server_address
                    .replace("http://", "")
                    .replace("https://", "");
            }

            let (address, port) = match server_address.find(':') {
                Some(index) => {
                    let (address, port) = server_address.split_at(index);

                    (address.to_owned(), port[1..].parse::<u16>().unwrap_or(3456))
                }
                None => (server_address, 3456),
            };

            let encryption: Option<ClientEncryptionType> = match tls {
                true => cert
                    .clone()
                    .map(|ca_cert_path| ClientEncryptionType::CustomTls { ca_cert_path })
                    .or_else(|| Some(ClientEncryptionType::NativeTls)),
                false => None,
            };

            let mut connection = match create_tcp_client(&address, port, encryption.clone()).await {
                Ok(connection) => connection,
                Err(e) => {
                    eprintln!("Failed to connect to server: {}", e);
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Failed to connect to server",
                    ));
                }
            };

            let ProcessConfigResponse::GetPublicEndpointConfig(endpoint_config) = connection
                .request_message(ProcessConfigRequest {
                    tunnel_key: key.clone(),
                    request: ConfigRequest::GetPublicEndpointConfig,
                })
                .await?;

            let mut tunnel_config = TunnelConfiguration {
                name: Some("my-tunnel".to_owned()),
                server_address: address.clone(),
                server_port: port,
                forward_connection_timeout_seconds: 5,
                encryption: encryption.into(),
                tunnel_key: key,
                monitor_key: None,
                proxies: Vec::new(),
            };

            for PublicEndpointConfig { name, config } in endpoint_config {
                match config {
                    PublicServerEndpointConfig::Http(http) => {
                        tunnel_config.proxies.push(TunnelProxy {
                            address: http.address.unwrap_or_else(|| address.clone()),
                            endpoint_name: name,
                            port: http.port,
                            endpoint_config: ProxyConfiguration::Http {
                                desired_name: http.allow_custom_hostnames.then(|| {
                                    http.hostname_template
                                        .replace("{name}", "custom_name")
                                        .replace("{port}", &http.port.to_string())
                                }),
                            },
                        });
                    }
                    PublicServerEndpointConfig::Tcp(tcp) => {
                        tunnel_config.proxies.push(TunnelProxy {
                            address: tcp.address.unwrap_or_else(|| address.clone()),
                            endpoint_name: name,
                            port: tcp.reserve_ports_from,
                            endpoint_config: ProxyConfiguration::Tcp {
                                desired_port: tcp
                                    .allow_desired_port
                                    .then(|| tcp.reserve_ports_from),
                            },
                        });
                    }
                    PublicServerEndpointConfig::Udp(udp) => {
                        tunnel_config.proxies.push(TunnelProxy {
                            address: udp.address.unwrap_or_else(|| address.clone()),
                            endpoint_name: name,
                            port: udp.reserve_ports_from,
                            endpoint_config: ProxyConfiguration::Udp {
                                desired_port: udp
                                    .allow_desired_port
                                    .then(|| udp.reserve_ports_from),
                                bind_address: None,
                            },
                        });
                    }
                }
            }

            write_configuration(tunnel_config.into())?;

            connection.shutdown().await;
        }
    }

    Ok(())
}

fn get_default_tunnel_configuration() -> TunnelConfiguration {
    let mut configuration = TunnelConfiguration {
        name: Some("my-tunnel".to_owned()),
        server_address: "tunnel-server.com".to_owned(),
        server_port: 3456,
        forward_connection_timeout_seconds: 5,
        encryption: Encryption::None,
        tunnel_key: Some("changethistunnelkey".to_owned()),
        monitor_key: Some("changethismonitorkey".to_owned()),
        proxies: Vec::new(),
    };

    configuration.proxies.push(TunnelProxy {
        address: "localhost".to_owned(),
        endpoint_name: "http".to_owned(),
        port: 8080,
        endpoint_config: ProxyConfiguration::Http { desired_name: None },
    });

    configuration
}

fn get_default_server_configuration() -> ServerConfiguration {
    let mut configuration = ServerConfiguration {
        server_port: 3456,
        server_address: None,
        monitor_key: Some("changethiskey".to_owned()),
        max_tunnel_input_wait: 30,
        tunnel_key: Some("changethistunnelkey".to_owned()),
        endpoints: HashMap::new(),
        max_tunnels: 50,
        max_clients: 100,
        max_proxies_per_tunnel: 10,
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
            reserve_ports_to: 4050,
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
            reserve_ports_to: 5050,
            inactivity_timeout: 20,
            full_hostname_template: Some("127.0.0.1:{port}".to_owned()),
            address: None,
        }),
    );

    configuration
}
