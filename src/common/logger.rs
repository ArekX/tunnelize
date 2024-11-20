use env_logger::Env;

use super::cli::Commands;
use std::io::Write;

#[cfg(debug_assertions)]
const VERBOSE_LOG_LEVEL: &str = "debug";

#[cfg(not(debug_assertions))]
const VERBOSE_LOG_LEVEL: &str = "info";

fn resolve_log_level(command: &Commands) -> &'static str {
    match command {
        Commands::Init { .. } => "error",
        Commands::Server { .. } => VERBOSE_LOG_LEVEL,
        Commands::Tunnel { verbose, .. } => {
            if *verbose {
                VERBOSE_LOG_LEVEL
            } else {
                "error"
            }
        }
        Commands::Monitor { .. } => "error",
    }
}

pub fn initialize_logger(command: &Commands) {
    let env = Env::default().filter_or("LOG_LEVEL", resolve_log_level(&command));

    let show_colors = std::env::var("LOG_COLORS").unwrap_or_else(|_| "true".to_string()) == "true";

    let mut builder = env_logger::Builder::from_env(env);

    builder.format(move |buf, record| {
        let level = match show_colors {
            true => match record.level() {
                log::Level::Error => "\x1b[31mERROR\x1b[0m",
                log::Level::Warn => "\x1b[33mWARN\x1b[0m",
                log::Level::Info => "\x1b[32mINFO\x1b[0m",
                log::Level::Debug => "\x1b[34mDEBUG\x1b[0m",
                log::Level::Trace => "\x1b[35mTRACE\x1b[0m",
            },
            false => match record.level() {
                log::Level::Error => "ERROR",
                log::Level::Warn => "WARN",
                log::Level::Info => "INFO",
                log::Level::Debug => "DEBUG",
                log::Level::Trace => "TRACE",
            },
        };

        #[cfg(debug_assertions)]
        {
            writeln!(
                buf,
                "[{}|{}|{}:{}] {}",
                level,
                chrono::Local::now().format("%H:%M:%S"),
                record.file().unwrap_or("unknown"),
                record.line().unwrap_or(0),
                record.args()
            )
        }

        #[cfg(not(debug_assertions))]
        {
            writeln!(
                buf,
                "[{}|{}] {}",
                level,
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.args()
            )
        }
    });

    builder.init();
}
