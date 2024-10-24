use clap::{command, Parser, Subcommand};

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
    #[command(subcommand)]
    Monitor(MonitorCommands),
}

#[derive(Subcommand, Debug, serde::Serialize, serde::Deserialize, Clone)]
pub enum MonitorCommands {
    SystemInfo,
}

pub fn parse_command() -> Commands {
    let args = Cli::parse();

    args.command.unwrap_or(Commands::Tunnel {
        init: false,
        #[cfg(debug_assertions)]
        verbose: true,
        #[cfg(not(debug_assertions))]
        verbose: false,
    })
}
