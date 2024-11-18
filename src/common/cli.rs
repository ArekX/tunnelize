use clap::{command, Parser, Subcommand};
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(
    name = "Tunnelize",
    author = "Aleksandar Panic",
    version,
    about = "Create and manage secure tunnels for HTTP, TCP and UDP traffic",
    long_about = "Tunnelize is a tool for creating secure tunnels between two endpoints for HTTP, TCP and UDP traffic."
)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Initialize tunnelize configuration")]
    Init {
        #[command(subcommand)]
        command: Option<InitCommands>,
    },
    #[command(about = "Start the Tunnelize server")]
    Server {
        #[arg(
            short = 'c',
            long,
            help = "Path to the server configuration file",
            long_help = "Specify a custom path to the server configuration file. If not provided, default configuration will be used."
        )]
        config: Option<String>,
    },
    #[command(about = "Start a tunnel to a remote tunnelize server")]
    Tunnel {
        #[arg(
            short = 'c',
            long,
            help = "Path to the tunnel configuration file",
            long_help = "Specify a custom path to the tunnel configuration file. If not provided, default configuration will be used."
        )]
        config: Option<String>,
        #[arg(
            long,
            short = 'v',
            default_value_t = false,
            help = "Enable verbose logging",
            long_help = "Enable detailed logging output for debugging purposes"
        )]
        verbose: bool,
    },
    #[command(about = "Monitor and manage running tunnels and connections")]
    Monitor {
        #[command(subcommand)]
        command: MonitorCommands,
        #[arg(
            short = 'c',
            long,
            help = "Path to the tunnel configuration file",
            long_help = "Specify a custom path to the tunnel configuration file for monitoring operations"
        )]
        config: Option<String>,
    },
}

#[derive(Subcommand, Debug, serde::Serialize, serde::Deserialize, Clone)]
pub enum InitCommands {
    #[command(
        name = "all",
        about = "Initialize default configuration for both server and tunnel",
        long_about = "Create configuration files for both server and tunnel in one command"
    )]
    All,
    #[command(about = "Initialize tunnel configuration")]
    Tunnel {
        #[arg(
            short = 's',
            long,
            help = "Server address to load config from",
            long_help = "Specify the address of the Tunnelize server to fetch configuration from"
        )]
        server: Option<String>,
        #[arg(
            short = 't',
            long,
            default_value = "false",
            help = "Use TLS to connect to server",
            long_help = "Enable TLS encryption for secure communication with the server"
        )]
        tls: bool,
        #[arg(
            short = 'c',
            long,
            help = "Path to the custom CA certificate file for TLS",
            long_help = "Specify a custom Certificate Authority certificate file for TLS verification"
        )]
        ca: Option<String>,
        #[arg(
            short = 'k',
            long,
            help = "Tunnel key to use to authenticate with server",
            long_help = "Provide an authentication key for secure tunnel registration with the server"
        )]
        key: Option<String>,
    },
    #[command(
        about = "Initialize server configuration",
        long_about = "Create a new configuration file for the Tunnelize server component"
    )]
    Server,
}

#[derive(Subcommand, Debug, serde::Serialize, serde::Deserialize, Clone)]
pub enum MonitorCommands {
    #[command(about = "Display system information and resources")]
    SystemInfo,
    #[command(about = "List all available endpoints")]
    ListEndpoints,
    #[command(about = "List all active tunnels")]
    ListTunnels,
    #[command(about = "Get detailed information about a specific tunnel")]
    GetTunnel { id: Uuid },
    #[command(about = "Disconnect and close a specific tunnel")]
    DisconnectTunnel { id: Uuid },
    #[command(about = "List all connected clients")]
    ListClients,
    #[command(about = "Get detailed information about a specific client")]
    GetClient { id: Uuid },
    #[command(about = "List all active tunnel links")]
    ListLinks,
    #[command(about = "Get detailed information about a specific link")]
    GetLink { id: Uuid },
    #[command(about = "Disconnect and close a specific link")]
    DisconnectLink { id: Uuid },
}

pub fn parse_command() -> Option<Commands> {
    Cli::parse().command
}
