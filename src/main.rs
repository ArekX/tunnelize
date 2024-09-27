use clap::{Parser, Subcommand};
use configuration::{
    get_default_server_config, get_default_tunnel_config, parse_configuration, write_tunnel_config,
    Configuration,
};
use env_logger::Env;
use log::{debug, error, info};

mod client;
mod configuration;
mod http;
mod server;
mod transport;

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

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let args = Args::parse();

    let command = args.command.unwrap_or(Commands::Tunnel { init: false });

    #[cfg(debug_assertions)]
    let env = Env::default()
        .filter_or("LOG_LEVEL", "trace")
        .write_style_or("LOG_STYLE", "always");

    #[cfg(not(debug_assertions))]
    let env = Env::default()
        .filter_or("LOG_LEVEL", "info")
        .write_style_or("LOG_STYLE", "always");

    env_logger::init_from_env(env);

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
                server::start_server(server).await?;
            } else {
                error!("No server configuration found, cannot start a server. Exiting...");
            }
        }
        Commands::Tunnel { init } => {
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
                if let Err(e) = client::start_server(tunnel).await {
                    debug!("Error starting tunnel client: {:?}", e);
                    error!("Could not start tunnel client due to error.");
                }
            } else {
                error!("No tunel configuration found, cannot start a tunnel. Exiting...");
            }
        }
    }

    Ok(())
}
