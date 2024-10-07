use log::debug;
use tokio::signal;
use tokio_util::sync::CancellationToken;

pub async fn start_cancel_listener(cancel_token: CancellationToken) {
    tokio::select! {
        _ = cancel_token.cancelled() => {
            debug!("Cancel token triggered.");
        }
        _ = signal::ctrl_c() => {
            debug!("Ctrl+C signal received.");
            cancel_token.cancel();
        }
    }
}
