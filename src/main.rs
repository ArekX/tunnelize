use common::{
    cli::{parse_command, Commands},
    logger::initialize_logger,
};
use log::{debug, info};
use serde::de;
use server::endpoints::monitor;

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
            return Ok(());
        }
        Commands::Server { init } => {
            if init {
                return Ok(());
            }

            info!("Starting server...");

            server::start().await?;
        }
        Commands::Tunnel { init, .. } => {
            if init {
                return Ok(());
            }

            tunnel::start().await?;
        }
        Commands::Monitor(monitor_command) => {
            tunnel::process_monitor_command(monitor_command).await?;
        }
    }

    Ok(())
}
