// use std::{fs::File, io::BufReader};

use std::{
    fs::File,
    io::{BufReader, Write},
};

use log::info;
use serde::{Deserialize, Serialize};

use crate::http::HttpServerConfig;

#[derive(Debug, Deserialize, Serialize)]
pub struct Configuration {
    pub server: Option<ServerConfiguration>,
    pub tunnel: Option<TunnelConfiguration>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfiguration {
    pub tunnel_port: u16,
    pub servers: Vec<ServerType>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerType {
    Http(HttpServerConfig),
    Tcp { port_range: (u16, u16) },
    Udp { port_range: (u16, u16) },
    MonitoringApi { port: u16 },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TunnelConfiguration {
    pub server_address: String,
    pub hostnames: Vec<HostnameConfiguration>,
    pub auth_key: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HostnameConfiguration {
    pub desired_name: Option<String>,
    pub forward_address: String,
}

fn get_configuration_dir() -> Result<std::path::PathBuf, std::io::Error> {
    let exe_dir = std::env::current_exe()?;
    let dir = exe_dir.parent().ok_or(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "Directory could not be found.",
    ))?;

    Ok(dir.to_owned())
}

pub fn configuration_exists() -> bool {
    let config_dir = get_configuration_dir().unwrap();
    let config_file = config_dir.join("tunnelize.json");

    config_file.exists()
}

pub fn parse_configuration() -> Result<Configuration, std::io::Error> {
    if !configuration_exists() {
        return Ok(Configuration {
            server: Some(ServerConfiguration {
                tunnel_port: 3456,
                servers: vec![ServerType::Http(HttpServerConfig {
                    client_port: 3457,
                    tunnel_port: 3456,
                    auth_key: None,
                    host_template: "t-{dynamic}.localhost".to_string(),
                })],
            }),
            tunnel: Some(TunnelConfiguration {
                server_address: "0.0.0.0:3456".to_string(),
                hostnames: vec![
                    HostnameConfiguration {
                        desired_name: Some("localhost:3457".to_string()),
                        forward_address: "0.0.0.0:8000".to_owned(),
                    },
                    HostnameConfiguration {
                        desired_name: Some("localhost:3457".to_string()),
                        forward_address: "0.0.0.0:3000".to_owned(),
                    },
                ],
                auth_key: None,
            }),
        });
    }

    let config_dir = get_configuration_dir()?;
    let config_file = config_dir.join("tunnelize.json");

    let file = File::open(config_file)?;
    let reader = BufReader::new(file);

    let config: Configuration = serde_json::from_reader(reader)?;

    Ok(config)
}

pub fn write_default_tunnel_config() -> Result<(), std::io::Error> {
    let config_dir = get_configuration_dir()?;
    let config_file = config_dir.join("tunnelize.json");

    let initial_config = serde_json::to_string_pretty(&Configuration {
        server: Some(ServerConfiguration {
            tunnel_port: 3456,
            servers: vec![ServerType::Http(HttpServerConfig {
                client_port: 3457,
                tunnel_port: 3456,
                auth_key: None,
                host_template: "t-{dynamic}.localhost".to_string(),
            })],
        }),
        tunnel: Some(TunnelConfiguration {
            server_address: "0.0.0.0:3456".to_string(),
            hostnames: vec![
                HostnameConfiguration {
                    desired_name: Some("8000".to_string()),
                    forward_address: "0.0.0.0:8000".to_owned(),
                },
                HostnameConfiguration {
                    desired_name: Some("3000".to_string()),
                    forward_address: "0.0.0.0:3000".to_owned(),
                },
            ],
            auth_key: None,
        }),
    })?;

    let mut file = File::create(config_file.clone())?;
    file.write_all(initial_config.as_bytes())?;

    info!("Default configuration written to {}", config_file.display());

    Ok(())
}
