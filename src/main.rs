use common::{
    cli::{parse_command, Commands},
    logger::initialize_logger,
};
use log::{debug, info};

mod common;
mod server;
pub mod tunnel;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let command = parse_command();

    initialize_logger(&command);

    if let Err(e) = run_command(command).await {
        debug!("Error running command: {:?}", e.to_string());
        std::process::exit(1);
    }

    Ok(())
}

async fn run_command(command: Commands) -> Result<(), std::io::Error> {
    match command {
        Commands::Init => {
            // write_tunnel_config(Configuration {
            //     server: Some(get_default_server_config()),
            //     tunnel: Some(get_default_tunnel_config()),
            // })?;
            return Ok(());
        }
        Commands::Server { init } => {
            if init {
                // write_tunnel_config(Configuration {
                //     server: Some(get_default_server_config()),
                //     tunnel: None,
                // })?;
                return Ok(());
            }

            // let config = get_configuration();

            info!("Starting server...");

            server::start().await?;

            // if let Some(server) = config.server {
            //     // server::start(server).await?;
            // } else {
            //     error!("No server configuration found, cannot start a server. Exiting...");
            // }
        }
        Commands::Tunnel { init, .. } => {
            if init {
                // write_tunnel_config(Configuration {
                //     server: None,
                //     tunnel: Some(get_default_tunnel_config()),
                // })?;
                return Ok(());
            }

            tunnel::start().await?;

            // let config = get_configuration();

            // info!("Starting client...");

            // if let Some(tunnel) = config.tunnel {
            //     // tunnel::start(tunnel).await?;
            // } else {
            //     error!("No tunel configuration found, cannot start a tunnel. Exiting...");
            // }
        }
    }

    Ok(())
}
