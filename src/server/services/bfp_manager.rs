use std::{collections::HashMap, net::IpAddr};

use chrono::Utc;

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
            bfp_ip_map: HashMap::new(),
        }
    }

    pub fn log_ip_attempt(&mut self, ip: &IpAddr) {
        let ip_string = ip.to_string();

        let attempt = match self.bfp_ip_map.get_mut(ip_string.as_str()) {
            Some(attempt) => attempt,
            None => {
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
            attempt.wait_until = Utc::now().timestamp() + 300;
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
