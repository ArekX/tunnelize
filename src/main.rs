use clap::{Parser, Subcommand};
use std::error::Error;

mod configuration;
mod proxy;
mod server;
mod server2;
mod ss;
mod sx;

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
    sx::start().await?;
    // let args = Args::parse();

    // let command = args.command.unwrap_or(Commands::Proxy);

    // match command {
    //     Commands::Server => {
    //         server2::start_server().await?;
    //     }
    //     Commands::Proxy => {
    //         proxy::start_proxy().await?;
    //     }
    // }

    Ok(())
}
