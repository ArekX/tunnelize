use std::{
    fs::File,
    io::{BufReader, BufWriter, ErrorKind},
};

use serde::{Deserialize, Serialize};

use tokio::io::Result;

use crate::{
    common::validate::{Validatable, Validation},
    server::configuration::ServerConfiguration,
    tunnel::configuration::TunnelConfiguration,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TunnelizeConfiguration {
    pub server: Option<ServerConfiguration>,
    pub tunnel: Option<TunnelConfiguration>,
}

impl Validatable for TunnelizeConfiguration {
    fn validate(&self, result: &mut Validation) {
        if let Some(server) = &self.server {
            result.validate_with_breadcrumb("server", server);
        }

        if let Some(tunnel) = &self.tunnel {
            result.validate_with_breadcrumb("tunnel", tunnel);
        }
    }
}

pub fn get_configuration_path() -> std::result::Result<std::path::PathBuf, std::io::Error> {
    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to get executable directory",
        )
    })?;

    Ok(exe_dir.join("tunnelize.json"))
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

pub fn load_configuration<T>() -> Result<T>
where
    T: TryFrom<TunnelizeConfiguration, Error = &'static str>,
{
    let config_path = get_configuration_path()?;
    let reader = BufReader::new(File::create(&config_path)?);

    let config: TunnelizeConfiguration = serde_json::from_reader(reader)?;

    let validation_result = Validation::validate(&config);

    if !validation_result.is_valid() {
        eprintln!("Configuration is invalid:");
        for error in validation_result.errors() {
            eprintln!("{}", error);
        }

        return Err(std::io::Error::new(
            ErrorKind::InvalidData,
            "Configuration is invalid.",
        ));
    }

    T::try_from(config).map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e))
}
