use clap::{Parser, Subcommand};
use configuration::{
    get_default_server_config, get_default_tunnel_config, parse_configuration, write_tunnel_config,
    Configuration,
};
use env_logger::Env;
use log::{debug, error, info};

mod configuration;
mod server;
mod transport;
mod tunnel;

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
    Init,
    Server {
        #[arg(long, default_value_t = false)]
        init: bool,
    },
    Tunnel {
        #[arg(long, default_value_t = false)]
        init: bool,
        #[arg(long, default_value_t = false)]
        verbose: bool,
    },
}

fn get_configuration() -> Configuration {
    match parse_configuration() {
        Ok(valid_config) => valid_config,
        Err(e) => {
            debug!("Error parsing configuration: {:?}", e);
            error!("Could not parse configuration file. Exiting...");
            std::process::exit(1);
        }
    }
}

#[cfg(debug_assertions)]
const VERBOSE_LOG_LEVEL: &str = "trace";

#[cfg(not(debug_assertions))]
const VERBOSE_LOG_LEVEL: &str = "info";

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let args = Args::parse();

    let command = args.command.unwrap_or(Commands::Tunnel {
        init: false,
        verbose: false,
    });

    let env = Env::default()
        .filter_or("LOG_LEVEL", resolve_log_level(&command))
        .write_style_or("LOG_STYLE", "always");

    env_logger::init_from_env(env);

    if let Err(e) = run_command(command).await {
        debug!("Error running command: {:?}", e.to_string());
        std::process::exit(1);
    }

    Ok(())
}

fn resolve_log_level(command: &Commands) -> &'static str {
    let verbose = match command {
        Commands::Tunnel { verbose, .. } => *verbose,
        _ => true,
    };

    if verbose {
        VERBOSE_LOG_LEVEL
    } else {
        "error"
    }
}

async fn run_command(command: Commands) -> Result<(), std::io::Error> {
    match command {
        Commands::Init => {
            write_tunnel_config(Configuration {
                server: Some(get_default_server_config()),
                tunnel: Some(get_default_tunnel_config()),
            })?;
            return Ok(());
        }
        Commands::Server { init } => {
            if init {
                write_tunnel_config(Configuration {
                    server: Some(get_default_server_config()),
                    tunnel: None,
                })?;
                return Ok(());
            }

            let config = get_configuration();

            info!("Starting server...");

            if let Some(server) = config.server {
                server::start(server).await?;
            } else {
                error!("No server configuration found, cannot start a server. Exiting...");
            }
        }
        Commands::Tunnel { init, .. } => {
            if init {
                write_tunnel_config(Configuration {
                    server: None,
                    tunnel: Some(get_default_tunnel_config()),
                })?;
                return Ok(());
            }

            let config = get_configuration();

            info!("Starting client...");

            if let Some(tunnel) = config.tunnel {
                tunnel::start(tunnel).await?;
            } else {
                error!("No tunel configuration found, cannot start a tunnel. Exiting...");
            }
        }
    }

    Ok(())
}
