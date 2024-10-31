use std::{collections::HashMap, time::Instant};

use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub struct ActivityTracker {
    client_map: HashMap<Uuid, Activity>,
}

pub struct Activity {
    pub cancel_token: CancellationToken,
    pub last_activity: Instant,
}

impl ActivityTracker {
    pub fn new() -> Self {
        Self {
            client_map: HashMap::new(),
        }
    }

    pub fn update_activity(&mut self, client_id: &Uuid) {
        if let Some(activity) = self.client_map.get_mut(client_id) {
            activity.last_activity = Instant::now();
        }
    }

    pub fn add(&mut self, cancel_token: CancellationToken) -> Uuid {
        let client_id = Uuid::new_v4();

        self.client_map.insert(
            client_id,
            Activity {
                cancel_token,
                last_activity: Instant::now(),
            },
        );

        client_id
    }

    pub fn cancel(&mut self, client_id: &Uuid) {
        if let Some(activity) = self.client_map.remove(client_id) {
            activity.cancel_token.cancel();
        }
    }

    pub async fn cancel_all_after_timeout(&mut self, timeout: u64) {
        self.client_map.retain(|_, activity| {
            if activity.last_activity.elapsed().as_secs() < timeout {
                true
            } else {
                activity.cancel_token.cancel();
                false
            }
        });
    }
}
