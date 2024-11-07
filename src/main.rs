use common::{
    cli::{parse_command, Commands},
    logger::initialize_logger,
};
use init::init_for;
use log::{debug, info};

mod common;
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
    match command {
        Commands::Init(commands) => {
            init_for(commands).await?;
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
