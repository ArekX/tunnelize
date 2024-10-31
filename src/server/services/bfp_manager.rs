use std::{collections::HashMap, net::IpAddr};

use chrono::Utc;

const MAX_IPS: usize = 1000;
const CLEANUP_OLD_SECONDS: i64 = 7200;
const BFP_WAIT_FOR_SECONDS: i64 = 300;

struct IpAttempt {
    count: u8,
    wait_until: i64,
}

pub struct BfpManager {
    bfp_ip_map: HashMap<String, IpAttempt>,
}

impl BfpManager {
    pub fn new() -> Self {
        Self {
            bfp_ip_map: HashMap::with_capacity(MAX_IPS),
        }
    }

    fn perform_cleanup(&mut self) {
        if self.bfp_ip_map.len() < MAX_IPS {
            return;
        }

        let now = Utc::now().timestamp();
        self.bfp_ip_map
            .retain(|_, attempt| now - attempt.wait_until <= CLEANUP_OLD_SECONDS);

        if self.bfp_ip_map.len() == MAX_IPS {
            let oldest_ip = self
                .bfp_ip_map
                .iter()
                .min_by_key(|&(_, attempt)| attempt.wait_until)
                .map(|(ip, _)| ip.clone());

            if let Some(ip) = oldest_ip {
                self.bfp_ip_map.remove(&ip);
            }
        }
    }

    pub fn log_ip_attempt(&mut self, ip: &IpAddr) {
        let ip_string = ip.to_string();

        let attempt = match self.bfp_ip_map.get_mut(ip_string.as_str()) {
            Some(attempt) => attempt,
            None => {
                self.perform_cleanup();

                self.bfp_ip_map.insert(
                    ip_string.clone(),
                    IpAttempt {
                        count: 0,
                        wait_until: 0,
                    },
                );

                self.bfp_ip_map.get_mut(ip_string.as_str()).unwrap()
            }
        };

        attempt.count = attempt.count.wrapping_add(1);
        if attempt.count >= 5 {
            attempt.wait_until = Utc::now().timestamp() + BFP_WAIT_FOR_SECONDS;
        }
    }

    pub fn is_locked(&mut self, ip: &IpAddr) -> bool {
        let ip_string = ip.to_string();

        match self.bfp_ip_map.get(ip_string.as_str()) {
            Some(attempt) => {
                return attempt.wait_until > Utc::now().timestamp();
            }
            None => false,
        }
    }

    pub fn clear_ip_attempts(&mut self, ip: &IpAddr) {
        self.bfp_ip_map.remove(ip.to_string().as_str());
    }
}
