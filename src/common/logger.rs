use env_logger::Env;

use super::cli::Commands;
use std::io::Write;
#[cfg(debug_assertions)]
const VERBOSE_LOG_LEVEL: &str = "trace";

#[cfg(not(debug_assertions))]
const VERBOSE_LOG_LEVEL: &str = "info";

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

pub fn initialize_logger(command: &Commands) {
    let env = Env::default()
        .filter_or("LOG_LEVEL", resolve_log_level(&command))
        .write_style_or("LOG_STYLE", "always");

    let mut builder = env_logger::Builder::from_env(env);

    #[cfg(not(debug_assertions))]
    builder.format_target(false);

    #[cfg(debug_assertions)]
    builder.format(|buf, record| {
        let level = match record.level() {
            log::Level::Error => "\x1b[31mERROR\x1b[0m",
            log::Level::Warn => "\x1b[33mWARN\x1b[0m",
            log::Level::Info => "\x1b[32mINFO\x1b[0m",
            log::Level::Debug => "\x1b[34mDEBUG\x1b[0m",
            log::Level::Trace => "\x1b[35mTRACE\x1b[0m",
        };

        writeln!(
            buf,
            "{}-{} [{}:{}] - {}",
            level,
            chrono::Local::now().format("%H:%M:%S"),
            record.file().unwrap_or("unknown"),
            record.line().unwrap_or(0),
            record.args()
        )
    });

    builder.init();
}
