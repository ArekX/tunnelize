// use std::{fs::File, io::BufReader};

use std::{
    fs::File,
    io::{BufReader, Write},
};

use log::{error, info};
use regex::Regex;
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
            client_authorize_user: None,
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

    if let Err(errors) = validate_configuration(&config) {
        for config_error in errors {
            error!("{}", config_error);
        }
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Configuration file contains errors.",
        ));
    }

    Ok(config)
}

pub fn validate_configuration(config: &Configuration) -> Result<(), Vec<String>> {
    let mut results: Vec<String> = vec![];

    let desired_name_regex = Regex::new(r"^[a-z0-9-]+$").unwrap();
    let ip_port_regex = Regex::new(r"^(?:[0-9]{1,3}\.){3}[0-9]{1,3}:[0-9]{1,5}$").unwrap();
    let hostname_port_regex = Regex::new(r"^[a-z0-9-]+:[0-9]{1,5}$").unwrap();

    if let Some(server) = &config.server {
        for server in server.servers.iter() {
            match server {
                ServerType::Http(http_config) => {
                    if http_config.client_port == http_config.tunnel_port {
                        results.push(
                            "Servers - HttpServer: Client and tunnel port cannot be the same."
                                .to_string(),
                        );
                    }

                    if http_config.host_template.is_empty()
                        || !http_config.host_template.contains("{dynamic}")
                    {
                        results.push("Servers - HttpServer: Host template cannot be empty and must contain {dynamic}.".to_string());
                    }
                }
                _ => {}
            }
        }
    }

    if let Some(tunnel) = &config.tunnel {
        if tunnel.server_address.is_empty()
            || (!ip_port_regex.is_match(&tunnel.server_address)
                && !hostname_port_regex.is_match(&tunnel.server_address))
        {
            results
                .push("Tunnel: Server address must be in the format '<ip>:<port>' or '<hostname>:<port>'.".to_string());
        }

        for hostname in &tunnel.hostnames {
            if hostname.forward_address.is_empty()
                || !ip_port_regex.is_match(&hostname.forward_address)
            {
                results.push(
                    "Tunnel - Hostnames: Forward address must be set and in the format '<ip>:<port>'."
                        .to_string(),
                );
            }

            if let Some(desired_name) = &hostname.desired_name {
                if desired_name.is_empty() || !desired_name_regex.is_match(desired_name) {
                    results.push(
                        "Tunnel - Hostnames: Desired name must be set and only contain lowercase alphanumeric characters and hyphens."
                            .to_string(),
                    );
                }
            }
        }
    }

    if results.is_empty() {
        return Ok(());
    }

    Err(results)
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
