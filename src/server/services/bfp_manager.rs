use std::{collections::HashMap, net::IpAddr};

use chrono::Utc;

const MAX_IPS: usize = 1000;
const CLEANUP_OLD_SECONDS: i64 = 7200;
const BFP_WAIT_FOR_SECONDS: i64 = 300;

struct IpAttempt {
    count: u8,
    wait_until: i64,
    added_at: i64,
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
            .retain(|_, attempt| now - attempt.added_at <= CLEANUP_OLD_SECONDS);

        if self.bfp_ip_map.len() == MAX_IPS {
            let oldest_ip = self
                .bfp_ip_map
                .iter()
                .min_by_key(|&(_, attempt)| attempt.added_at)
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
                        added_at: Utc::now().timestamp(),
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
                attempt.wait_until > Utc::now().timestamp()
            }
            None => false,
        }
    }

    pub fn clear_ip_attempts(&mut self, ip: &IpAddr) {
        self.bfp_ip_map.remove(ip.to_string().as_str());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::IpAddr;
    use std::str::FromStr;

    fn get_test_ip(ip_str: &str) -> IpAddr {
        IpAddr::from_str(ip_str).unwrap()
    }

    #[test]
    fn test_log_ip_attempt() {
        let mut manager = BfpManager::new();
        let ip = get_test_ip("192.168.0.1");

        for _ in 0..5 {
            manager.log_ip_attempt(&ip);
        }

        assert!(manager.is_locked(&ip));
    }

    #[test]
    fn test_is_locked() {
        let mut manager = BfpManager::new();
        let ip = get_test_ip("192.168.0.2");

        assert!(!manager.is_locked(&ip));

        for _ in 0..5 {
            manager.log_ip_attempt(&ip);
        }

        assert!(manager.is_locked(&ip));
    }

    #[test]
    fn test_clear_ip_attempts() {
        let mut manager = BfpManager::new();
        let ip = get_test_ip("192.168.0.3");

        for _ in 0..5 {
            manager.log_ip_attempt(&ip);
        }

        assert!(manager.is_locked(&ip));

        manager.clear_ip_attempts(&ip);

        assert!(!manager.is_locked(&ip));
    }

    #[test]
    fn test_perform_cleanup() {
        let mut manager = BfpManager::new();
        let ip = get_test_ip("192.199.233.4");

        let mut added_ips: Vec<IpAddr> = vec![];

        let mut left_value = 0;
        for i in 0..MAX_IPS {
            let right_value = i % 255;

            if right_value == 0 {
                left_value += 1;
            }

            let ip = get_test_ip(&format!("192.168.{left_value}.{right_value}"));

            added_ips.push(ip);

            manager.log_ip_attempt(&ip);
            assert!(manager.bfp_ip_map.contains_key(&ip.to_string()));
        }
        assert_eq!(manager.bfp_ip_map.len(), MAX_IPS);
        manager.log_ip_attempt(&ip);
        assert_eq!(manager.bfp_ip_map.len(), MAX_IPS);
        
        let remaining_ips: Vec<_> = manager.bfp_ip_map.keys().cloned().collect();
        let difference: Vec<_> = added_ips
            .iter()
            .map(|ip| ip.to_string())
            .filter(|ip| !remaining_ips.contains(ip))
            .collect();

        assert_eq!(difference.len(), 1);
    }
}
