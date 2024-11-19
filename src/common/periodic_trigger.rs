use std::time::Duration;

use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio_util::sync::CancellationToken;

pub struct PeriodicTrigger {
    interval: Duration,
    tx: Sender<()>,
    cancel_token: CancellationToken,
}

impl PeriodicTrigger {
    pub fn new(interval: Duration) -> (Self, Receiver<()>) {
        let (tx, rx) = channel::<()>(100);
        (
            Self {
                interval,
                tx,
                cancel_token: CancellationToken::new(),
            },
            rx,
        )
    }

    pub fn start(&self) {
        let cancel_token = self.cancel_token.clone();
        let tx = self.tx.clone();
        let interval = self.interval;

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        break;
                    }
                    _ = tokio::time::sleep(interval) => {
                        tx.send(()).await.unwrap();
                    }
                }
            }
        });
    }

    pub fn cancel(&self) {
        self.cancel_token.cancel();
    }
}
