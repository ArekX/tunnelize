use clap::{command, Parser, Subcommand};
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(
    name = "Tunnelize",
    author = "Aleksandar Panic",
    version,
    long_about = "Tunnelize is a tool for creating secure tunnels between two endpoints for HTTP, TCP and UDP traffic."
)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Init {
        #[command(subcommand)]
        command: Option<InitCommands>,
    },
    Server {
        #[arg(short = 'c', long, help = "Path to the server configuration file")]
        config: Option<String>,
    },
    Tunnel {
        #[arg(short = 'c', long, help = "Path to the tunnel configuration file")]
        config: Option<String>,
        #[arg(long, short = 'v', default_value_t = false)]
        verbose: bool,
    },
    Monitor {
        #[command(subcommand)]
        command: MonitorCommands,
        #[arg(short = 'c', long, help = "Path to the tunnel configuration file")]
        config: Option<String>,
    },
}

#[derive(Subcommand, Debug, serde::Serialize, serde::Deserialize, Clone)]
pub enum InitCommands {
    #[command(name = "all")]
    All,
    Tunnel {
        #[arg(short = 's', long, help = "Server address to load config from")]
        server: Option<String>,
        #[arg(
            short = 't',
            long,
            default_value = "false",
            help = "Use TLS to connect to server"
        )]
        tls: bool,
        #[arg(
            short = 'c',
            long,
            help = "Path to the custom CA certificate file for TLS"
        )]
        cert: Option<String>,
        #[arg(
            short = 'k',
            long,
            help = "Tunnel key to use to authenticate with server"
        )]
        key: Option<String>,
    },
    Server,
}

#[derive(Subcommand, Debug, serde::Serialize, serde::Deserialize, Clone)]
pub enum MonitorCommands {
    SystemInfo,
    ListEndpoints,
    ListTunnels,
    GetTunnel { id: Uuid },
    DisconnectTunnel { id: Uuid },
    ListClients,
    GetClient { id: Uuid },
    ListLinks,
    GetLink { id: Uuid },
    DisconnectLink { id: Uuid },
}

pub fn parse_command() -> Commands {
    let args = Cli::parse();

    args.command.unwrap_or(Commands::Tunnel {
        config: None,
        #[cfg(debug_assertions)]
        verbose: true,
        #[cfg(not(debug_assertions))]
        verbose: false,
    })
}
