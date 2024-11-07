use clap::{command, Parser, Subcommand};
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(
    name = "Tunnelize",
    author = "Aleksandar Panic",
    version,
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(subcommand)]
    Init(InitCommands),
    Server,
    Tunnel {
        #[arg(long, short = 'v', default_value_t = false)]
        verbose: bool,
    },
    #[command(subcommand)]
    Monitor(MonitorCommands),
}

#[derive(Subcommand, Debug, serde::Serialize, serde::Deserialize, Clone)]
pub enum InitCommands {
    Tunnel {
        #[arg(short = 's', long)]
        server: Option<String>,
        #[arg(short = 'c', long)]
        cert: Option<String>,
        #[arg(short = 'k', long)]
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
        #[cfg(debug_assertions)]
        verbose: true,
        #[cfg(not(debug_assertions))]
        verbose: false,
    })
}
