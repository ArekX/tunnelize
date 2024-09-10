use clap::{Parser, Subcommand};
use configuration::parse_configuration;
use env_logger::Env;
use log::{error, info};
use std::error::Error;

mod configuration;
mod messages;
mod server;
mod tunnel;

/// CLI interpreter for Brain**** language
#[derive(Parser, Debug)]
#[command(
    name = "Tunnelize", 
    author = "Aleksandar Panic", 
    version,
    long_about = None
)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Server,
    Proxy,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // testv2::start_all_this().await?;
    let args = Args::parse();

    let command = args.command.unwrap_or(Commands::Proxy);

    let env = Env::default()
        .filter_or("LOG_LEVEL", "trace")
        .write_style_or("LOG_STYLE", "always");

    env_logger::init_from_env(env);

    match command {
        Commands::Server => {
            info!("Starting server...");

            let config = parse_configuration()?;

            if let Some(server) = config.server {
                server::start_server(server).await?;
            } else {
                error!("No server configuration found. Exiting...");
            }
        }
        Commands::Proxy => {
            info!("Starting client...");

            let config = parse_configuration()?;

            if let Some(tunnel) = config.tunnel {
                tunnel::start_client(tunnel).await?;
            } else {
                error!("No tunel configuration found. Exiting...");
            }
        }
    }

    Ok(())
}
