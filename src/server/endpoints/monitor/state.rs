use std::{
    collections::HashMap,
    net::IpAddr,
    sync::{atomic::AtomicU8, Arc},
};

use chrono::Utc;
use tokio::sync::Mutex;

use crate::server::services::Services;

use super::configuration::MonitorEndpointConfig;

struct IpAttempt {
    count: u8,
    wait_until: i64,
}

#[derive(Clone)]
pub struct AppState {
    pub services: Arc<Services>,
    pub config: Arc<MonitorEndpointConfig>,
    pub name: String,
    bfp_ip_map: Arc<Mutex<HashMap<String, IpAttempt>>>,
}

impl AppState {
    pub fn new(services: Arc<Services>, config: Arc<MonitorEndpointConfig>, name: String) -> Self {
        Self {
            services,
            config,
            name,
            bfp_ip_map: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn log_ip_attempt(&self, ip: &IpAddr) {
        let mut map = self.bfp_ip_map.lock().await;
        let ip_string = ip.to_string();

        let attempt = match map.get_mut(ip_string.as_str()) {
            Some(attempt) => attempt,
            None => {
                map.insert(
                    ip_string.clone(),
                    IpAttempt {
                        count: 0,
                        wait_until: 0,
                    },
                );

                map.get_mut(ip_string.as_str()).unwrap()
            }
        };

        attempt.count = attempt.count.wrapping_add(1);
        if attempt.count >= 5 {
            attempt.wait_until = Utc::now().timestamp() + 300;
        }
    }

    pub async fn is_locked(&self, ip: &IpAddr) -> bool {
        let map = self.bfp_ip_map.lock().await;
        let ip_string = ip.to_string();

        match map.get(ip_string.as_str()) {
            Some(attempt) => {
                return attempt.wait_until > Utc::now().timestamp();
            }
            None => false,
        }
    }

    pub async fn clear_ip_attempts(&self, ip: &IpAddr) {
        let mut map = self.bfp_ip_map.lock().await;
        let ip_string = ip.to_string();

        map.remove(ip_string.as_str());
    }
}
