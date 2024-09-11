// use std::{fs::File, io::BufReader};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Configuration {
    pub server: Option<ServerConfiguration>,
    pub tunnel: Option<TunnelConfiguration>,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfiguration {
    pub tunnel_address: String,
    pub client_address: String,
}

#[derive(Debug, Deserialize)]
pub struct TunnelConfiguration {
    pub server_address: String,
    pub hostname: String,
}

// fn get_configuration_dir() -> Result<std::path::PathBuf, std::io::Error> {
//     let exe_dir = std::env::current_exe()?;
//     let dir = exe_dir.parent().ok_or(std::io::Error::new(
//         std::io::ErrorKind::NotFound,
//         "Directory could not be found.",
//     ))?;

//     Ok(dir.to_owned())
// }

// pub fn configuration_exists() -> bool {
//     let config_dir = get_configuration_dir().unwrap();
//     let config_file = config_dir.join("tunnelize.json");

//     config_file.exists()
// }

pub fn parse_configuration() -> Result<Configuration, std::io::Error> {
    Ok(Configuration {
        server: Some(ServerConfiguration {
            tunnel_address: "0.0.0.0:3456".to_string(),
            client_address: "0.0.0.0:3457".to_string(),
        }),
        tunnel: Some(TunnelConfiguration {
            server_address: "arekxv.name:3456".to_string(),
            hostname: "tunnel-opa.arekxv.name".to_string(),
        }),
    })
}
