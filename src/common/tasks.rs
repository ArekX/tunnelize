use log::debug;
use tokio::signal;
use tokio_util::sync::CancellationToken;

pub async fn start_cancel_listener(cancel_token: CancellationToken) {
    if let Err(e) = signal::ctrl_c().await {
        debug!("Error while waiting for ctrl+c signal: {:?}", e);
        return;
    }

    cancel_token.cancel();
}
