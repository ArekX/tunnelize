use std::{
    fs::File,
    io::{BufReader, BufWriter, ErrorKind},
    path::PathBuf,
};

use log::info;
use serde::{Deserialize, Serialize};

use serde_json::Value;
use tokio::io::Result;

use crate::{
    common::{
        cli::Commands,
        validate::{Validatable, Validation},
    },
    server::configuration::ServerConfiguration,
    tunnel::configuration::TunnelConfiguration,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TunnelizeConfiguration {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub server: Option<ServerConfiguration>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub tunnel: Option<TunnelConfiguration>,
    // TODO: Add logging configuration to enable/disable loggino and set log level and colors
    // TODO: Make more sensible defaults for server and tunnel configurations
}

impl Validatable for TunnelizeConfiguration {
    fn validate(&self, result: &mut Validation) {
        if let Some(server) = &self.server {
            result.validate_child("server", server);
        }

        if let Some(tunnel) = &self.tunnel {
            result.validate_child("tunnel", tunnel);
        }
    }
}

pub fn get_configuration_path() -> std::result::Result<std::path::PathBuf, std::io::Error> {
    let path = PathBuf::from("tunnelize.json");
    Ok(path)
}

pub fn write_configuration(configuration: TunnelizeConfiguration) -> Result<()> {
    let config_path = get_configuration_path()?;

    serde_json::to_writer_pretty(BufWriter::new(File::create(&config_path)?), &configuration)?;

    println!(
        "Initialized tunnel configuration at {}",
        config_path.to_str().unwrap_or("<unknown>")
    );

    Ok(())
}

pub fn load_configuration<T>(config_file: Option<String>) -> Result<T>
where
    T: TryFrom<TunnelizeConfiguration, Error = &'static str>,
{
    let config_path = config_file
        .map(|f| Ok::<PathBuf, std::io::Error>(PathBuf::from(f)))
        .unwrap_or_else(|| Ok::<PathBuf, std::io::Error>(get_configuration_path()?))?;

    if !config_path.exists() {
        println!(
            "Configuration file not found at '{}'. Please run init command first.",
            config_path.to_str().unwrap_or("<unknown>")
        );
        return Err(std::io::Error::new(
            ErrorKind::NotFound,
            "Configuration file not found.",
        ));
    }

    info!(
        "Loading configuration from {}",
        config_path.to_str().unwrap_or("<unknown>")
    );
    let reader = BufReader::new(File::open(&config_path)?);

    let config: TunnelizeConfiguration = serde_json::from_reader(reader)?;

    let validation_result = Validation::validate(&config);

    if !validation_result.is_valid() {
        eprintln!("Configuration is invalid. Please fix following errors:");
        for error in validation_result.errors() {
            eprintln!("- {}", error);
        }

        return Err(std::io::Error::new(
            ErrorKind::InvalidData,
            "Configuration is invalid.",
        ));
    }

    T::try_from(config).map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e))
}

pub fn get_default_command() -> Commands {
    let tunnel = Commands::Tunnel {
        config: None,
        #[cfg(debug_assertions)]
        verbose: true,
        #[cfg(not(debug_assertions))]
        verbose: false,
    };

    let Ok(config_path) = get_configuration_path() else {
        return tunnel;
    };

    let Ok(file) = File::open(config_path) else {
        return tunnel;
    };

    let reader = BufReader::new(file);
    let Ok(json) = serde_json::from_reader::<_, Value>(reader) else {
        return tunnel;
    };

    let mut server_key_set = false;
    let mut tunnel_key_set = false;

    if let Some(obj) = json.as_object() {
        for (key, value) in obj.iter().take(2) {
            if key == "server" && !value.is_null() {
                server_key_set = true;
            } else if key == "tunnel" && !value.is_null() {
                tunnel_key_set = true;
            }
        }
    }

    if server_key_set && !tunnel_key_set {
        return Commands::Server { config: None };
    }

    return tunnel;
}
