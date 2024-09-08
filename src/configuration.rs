use std::{fs::File, io::BufReader};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub tunnel_address: Option<String>,
    pub client_address: Option<String>,
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
    let config_dir = get_configuration_dir()?;
    let file = File::open(config_dir.join("tunnelize.json"))?;
    let mut reader = BufReader::new(file);

    Ok(serde_json::from_reader(&mut reader)?)
}
