use env_logger::Env;

use super::cli::Commands;

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

    builder.init();
}
