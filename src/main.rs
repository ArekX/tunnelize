use common::{
    cli::{parse_command, Commands, InitCommands},
    logger::initialize_logger,
};
use init::init_for;
use log::{debug, info};

mod common;
pub mod configuration;
mod init;
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
    // TODO: Server/Tunnel load and validate config
    match command {
        Commands::Init { command } => {
            init_for(command.unwrap_or_else(|| InitCommands::All)).await?;
        }
        Commands::Server => {
            info!("Starting server...");

            server::start().await?;
        }
        Commands::Tunnel { .. } => {
            tunnel::start().await?;
        }
        Commands::Monitor(monitor_command) => {
            tunnel::process_monitor_command(monitor_command).await?;
        }
    }

    Ok(())
}
