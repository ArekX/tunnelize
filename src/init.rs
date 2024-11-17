use std::collections::HashMap;

use log::error;

use crate::{
    common::{
        cli::InitCommands,
        tcp_client::{create_tcp_client, ClientEncryption},
    },
    configuration::{write_configuration, TunnelizeConfiguration},
    server::{
        configuration::{EndpointConfiguration, ServerConfiguration},
        endpoints::{
            http::configuration::HttpEndpointConfig,
            monitor::configuration::{MonitorAuthentication, MonitorEndpointConfig},
            tcp::configuration::TcpEndpointConfig,
            udp::configuration::UdpEndpointConfig,
        },
        incoming_requests::{
            ConfigRequest, ProcessConfigRequest, ProcessConfigResponse, PublicEndpointConfig,
            PublicServerEndpointConfig,
        },
    },
    tunnel::configuration::{ProxyConfiguration, TunnelConfiguration, TunnelProxy},
};

pub async fn init_for(command: InitCommands) -> Result<(), std::io::Error> {
    match command {
        InitCommands::All => {
            write_configuration(TunnelizeConfiguration {
                server: Some(get_default_server_configuration().into()),
                tunnel: Some(get_default_tunnel_configuration().into()),
            })?;
        }
        InitCommands::Server => {
            write_configuration(get_default_server_configuration().into())?;
        }
        InitCommands::Tunnel {
            server,
            ca: ca_path,
            tls,
            key,
        } => {
            let Some(mut server_address) = server else {
                write_configuration(get_default_tunnel_configuration().into())?;

                return Ok(());
            };

            println!("Connecting to: {}", server_address);

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

            let encryption: Option<ClientEncryption> =
                tls.then(|| ClientEncryption::Tls { ca_path });

            let mut connection =
                match create_tcp_client(&address, port, encryption.clone().into()).await {
                    Ok(connection) => connection,
                    Err(e) => {
                        error!("Could not retrieve configuration: {}", e);
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "Failed to connect to server",
                        ));
                    }
                };

            let endpoint_config = match connection
                .request_message(ProcessConfigRequest {
                    tunnel_key: key.clone(),
                    request: ConfigRequest::GetPublicEndpointConfig,
                })
                .await?
            {
                ProcessConfigResponse::GetPublicEndpointConfig(config) => config,
                ProcessConfigResponse::AccessDenied => {
                    error!("Could not retrieve configuration: Access denied, please check your tunnel key.");
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        "Access denied. Please check your tunnel key.",
                    ));
                }
            };

            let mut tunnel_config = TunnelConfiguration {
                name: Some("my-tunnel".to_owned()),
                server_address: address.clone(),
                server_port: None,
                forward_connection_timeout_seconds: None,
                encryption: Some(encryption.into()),
                tunnel_key: key,
                monitor_key: None,
                proxies: Vec::new(),
            };

            for PublicEndpointConfig { name, config } in endpoint_config {
                match config {
                    PublicServerEndpointConfig::Http(http) => {
                        tunnel_config.proxies.push(TunnelProxy {
                            address: "localhost".to_owned(),
                            endpoint_name: name,
                            port: http.port,
                            endpoint_config: ProxyConfiguration::Http { desired_name: None },
                        });
                    }
                    PublicServerEndpointConfig::Tcp(tcp) => {
                        tunnel_config.proxies.push(TunnelProxy {
                            address: "localhost".to_owned(),
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
                            address: "localhost".to_owned(),
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
        server_address: "localhost".to_owned(),
        server_port: None,
        forward_connection_timeout_seconds: None,
        encryption: None,
        tunnel_key: None,
        monitor_key: Some("changethismonitorkey".to_owned()),
        proxies: Vec::new(),
    };

    configuration.proxies.push(TunnelProxy {
        address: "localhost".to_owned(),
        endpoint_name: "http".to_owned(),
        port: 8080,
        endpoint_config: ProxyConfiguration::Http { desired_name: None },
    });

    configuration.proxies.push(TunnelProxy {
        address: "localhost".to_owned(),
        endpoint_name: "tcp".to_owned(),
        port: 4000,
        endpoint_config: ProxyConfiguration::Tcp { desired_port: None },
    });

    configuration.proxies.push(TunnelProxy {
        address: "localhost".to_owned(),
        endpoint_name: "udp".to_owned(),
        port: 5000,
        endpoint_config: ProxyConfiguration::Udp {
            desired_port: None,
            bind_address: None,
        },
    });

    configuration
}

fn get_default_server_configuration() -> ServerConfiguration {
    let mut configuration = ServerConfiguration {
        server_port: None,
        server_address: None,
        monitor_key: Some("changethismonitorkey".to_owned()),
        max_tunnel_input_wait: None,
        tunnel_key: None,
        endpoints: HashMap::new(),
        max_tunnels: None,
        max_clients: None,
        max_proxies_per_tunnel: None,
        encryption: None,
    };

    configuration.endpoints.insert(
        "http".to_owned(),
        EndpointConfiguration::Http(HttpEndpointConfig {
            port: 3457,
            encryption: None,
            address: None,
            max_client_input_wait_secs: None,
            hostname_template: "tunnel-{name}.localhost".to_owned(),
            full_url_template: None,
            allow_custom_hostnames: None,
            require_authorization: None,
        }),
    );

    configuration.endpoints.insert(
        "monitor".to_owned(),
        EndpointConfiguration::Monitoring(MonitorEndpointConfig {
            encryption: None,
            port: 3000,
            address: None,
            allow_cors_origins: None,
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
            allow_desired_port: None,
            encryption: None,
            full_hostname_template: Some("localhost:{port}".to_owned()),
            address: None,
        }),
    );

    configuration.endpoints.insert(
        "udp".to_owned(),
        EndpointConfiguration::Udp(UdpEndpointConfig {
            reserve_ports_from: 5000,
            allow_desired_port: None,
            reserve_ports_to: 5050,
            inactivity_timeout: None,
            full_hostname_template: Some("localhost:{port}".to_owned()),
            address: None,
        }),
    );

    configuration
}
