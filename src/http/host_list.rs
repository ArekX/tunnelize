use std::collections::HashMap;

use rand::Rng;
use uuid::Uuid;

const CHARACTER_SET: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
fn generate_random_string() -> String {
    let mut rng = rand::thread_rng();

    (0..5)
        .map(|_| {
            let idx = rng.gen_range(0..CHARACTER_SET.len());
            CHARACTER_SET[idx] as char
        })
        .collect()
}

pub struct ResolvedHost {
    pub host_id: Uuid,
    pub hostname: String,
}

#[derive(Clone)]
pub struct RegisteredHost {
    pub tunnel_id: Uuid,
    pub host_id: Uuid,
    // pub hostname: String,
}

pub struct HostList {
    host_map: HashMap<String, RegisteredHost>,
    allow_custom_hostnames: bool,
    host_template: String,
}

impl HostList {
    pub fn new(host_template: String, allow_custom_hostnames: bool) -> Self {
        HostList {
            host_template,
            allow_custom_hostnames,
            host_map: HashMap::new(),
        }
    }

    fn assign_hostname(&self, desired_name: Option<String>) -> String {
        if self.allow_custom_hostnames {
            if let Some(name) = desired_name {
                if name.len() >= 1 {
                    let hostname = self.host_template.replace("{dynamic}", &name);

                    if !self.host_map.contains_key(&hostname) {
                        return hostname;
                    }
                }
            }
        }

        let mut hostname = self
            .host_template
            .replace("{dynamic}", &generate_random_string());

        while self.host_map.contains_key(&hostname) {
            hostname = self
                .host_template
                .replace("{dynamic}", &generate_random_string());
        }

        hostname
    }

    pub fn register(&mut self, tunnel_id: Uuid, preferred_name: Option<String>) -> ResolvedHost {
        let hostname = self.assign_hostname(preferred_name);
        let host_id = Uuid::new_v4();

        self.host_map.insert(
            hostname.clone(),
            RegisteredHost {
                tunnel_id,
                host_id,
                // hostname: hostname.clone(),
            },
        );

        ResolvedHost { host_id, hostname }
    }

    pub fn unregister_by_tunnel(&mut self, tunnel_id: Uuid) {
        let mut keys_to_remove = Vec::new();

        for (key, value) in self.host_map.iter() {
            if value.tunnel_id == tunnel_id {
                keys_to_remove.push(key.clone());
            }
        }

        for key in keys_to_remove {
            self.host_map.remove(&key);
        }
    }

    pub fn find_host(&self, hostname: &String) -> Option<RegisteredHost> {
        if let Some(host) = self.host_map.get(hostname) {
            return Some(host.clone());
        }

        None
    }
}
