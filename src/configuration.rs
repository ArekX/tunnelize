// use std::{fs::File, io::BufReader};

use std::{fs::File, io::Write};

use log::info;
use serde::{Deserialize, Serialize};

use crate::servers::http::HttpServer;

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
    Http(HttpServer),
    Tcp { port_range: (u16, u16) },
    Udp { port_range: (u16, u16) },
    MonitoringApi { port: u16 },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TunnelConfiguration {
    pub server_address: String,
    pub hostnames: Vec<HostnameConfiguration>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HostnameConfiguration {
    pub name: Option<String>,
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

// pub fn configuration_exists() -> bool {
//     let config_dir = get_configuration_dir().unwrap();
//     let config_file = config_dir.join("tunnelize.json");

//     config_file.exists()
// }

pub fn parse_configuration() -> Result<Configuration, std::io::Error> {
    Ok(Configuration {
        server: Some(ServerConfiguration {
            tunnel_port: 3456,
            servers: vec![ServerType::Http(HttpServer {
                port: 3457,
                auth_key: None,
            })],
        }),
        tunnel: Some(TunnelConfiguration {
            server_address: "0.0.0.0:3456".to_string(),
            hostnames: vec![
                HostnameConfiguration {
                    name: Some("localhost:3457".to_string()),
                    forward_address: "0.0.0.0:8000".to_owned(),
                },
                HostnameConfiguration {
                    name: Some("localhost:3457".to_string()),
                    forward_address: "0.0.0.0:3000".to_owned(),
                },
            ],
        }),
    })
}

pub fn write_default_tunnel_config() -> Result<(), std::io::Error> {
    let config_dir = get_configuration_dir()?;
    let config_file = config_dir.join("tunnelize.json");

    let initial_config = serde_json::to_string_pretty(&Configuration {
        server: Some(ServerConfiguration {
            tunnel_port: 3456,
            servers: vec![ServerType::Http(HttpServer {
                port: 3457,
                auth_key: None,
            })],
        }),
        tunnel: Some(TunnelConfiguration {
            server_address: "0.0.0.0:3456".to_string(),
            hostnames: vec![
                HostnameConfiguration {
                    name: Some("localhost:3457".to_string()),
                    forward_address: "0.0.0.0:8000".to_owned(),
                },
                HostnameConfiguration {
                    name: Some("localhost:3457".to_string()),
                    forward_address: "0.0.0.0:3000".to_owned(),
                },
            ],
        }),
    })?;

    let mut file = File::create(config_file.clone())?;
    file.write_all(initial_config.as_bytes())?;

    info!("Default configuration written to {}", config_file.display());

    Ok(())
}
