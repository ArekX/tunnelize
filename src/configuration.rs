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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<ServerConfiguration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tunnel: Option<TunnelConfiguration>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfiguration {
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

pub fn get_default_server_config() -> ServerConfiguration {
    ServerConfiguration {
        servers: vec![ServerType::Http(HttpServerConfig {
            client_port: 3457,
            tunnel_port: 3456,
            auth_key: None,
            host_template: "t-{dynamic}.localhost".to_string(),
            allow_custom_hostnames: true,
        })],
    }
}

pub fn get_default_tunnel_config() -> TunnelConfiguration {
    TunnelConfiguration {
        server_address: "0.0.0.0:3456".to_string(),
        hostnames: vec![HostnameConfiguration {
            desired_name: Some("8000".to_string()),
            forward_address: "0.0.0.0:8000".to_owned(),
        }],
        auth_key: None,
    }
}

pub fn parse_configuration() -> Result<Configuration, std::io::Error> {
    if !configuration_exists() {
        return Ok(Configuration {
            server: Some(get_default_server_config()),
            tunnel: Some(get_default_tunnel_config()),
        });
    }

    let config_dir = get_configuration_dir()?;
    let config_file = config_dir.join("tunnelize.json");

    let file = File::open(config_file)?;
    let reader = BufReader::new(file);

    let config: Configuration = serde_json::from_reader(reader)?;

    Ok(config)
}

pub fn write_tunnel_config(config: Configuration) -> Result<(), std::io::Error> {
    let config_dir = get_configuration_dir()?;
    let config_file = config_dir.join("tunnelize.json");

    let initial_config = serde_json::to_string_pretty(&config)?;

    let mut file = File::create(config_file.clone())?;
    file.write_all(initial_config.as_bytes())?;

    info!("Default configuration written to {}", config_file.display());

    Ok(())
}
