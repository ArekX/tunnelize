use common::{
    cli::{parse_command, Commands, InitCommands},
    logger::initialize_logger,
};
use configuration::get_default_command;
use init::init_for;
use log::{debug, info};

mod common;
pub mod configuration;
mod init;
mod server;
pub mod tunnel;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let command = match parse_command() {
        Some(command) => command,
        None => get_default_command(),
    };

    initialize_logger(&command);

    println!("{:?}", command);

    if let Err(e) = run_command(command).await {
        debug!("Error running command: {:?}", e.to_string());
        std::process::exit(1);
    }

    Ok(())
}

async fn run_command(command: Commands) -> Result<(), std::io::Error> {
    match command {
        Commands::Init { command } => {
            init_for(command.unwrap_or_else(|| InitCommands::All)).await?;
        }
        Commands::Server { config } => {
            info!("Starting server...");

            server::start(config).await?;
        }
        Commands::Tunnel { config, .. } => {
            tunnel::start(config).await?;
        }
        Commands::Monitor { command, config } => {
            tunnel::process_monitor_command(command, config).await?;
        }
    }

    Ok(())
}
