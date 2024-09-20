use std::collections::HashMap;

use rand::Rng;

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
    pub host_id: u32,
    pub hostname: String,
}

#[derive(Clone)]
pub struct RegisteredHost {
    pub tunnel_id: u32,
    pub host_id: u32,
    // pub hostname: String,
}

pub struct HostList {
    id_counter: u32,
    host_map: HashMap<String, RegisteredHost>,
    host_template: String,
}

impl HostList {
    pub fn new(host_template: String) -> Self {
        HostList {
            id_counter: 0,
            host_template,
            host_map: HashMap::new(),
        }
    }

    fn assign_id(&mut self) -> u32 {
        self.id_counter = self.id_counter.wrapping_add(1);
        self.id_counter
    }

    fn assign_hostname(&self, desired_name: Option<String>) -> String {
        if let Some(name) = desired_name {
            if name.len() >= 1 {
                let hostname = self.host_template.replace("{dynamic}", &name);

                if !self.host_map.contains_key(&hostname) {
                    return hostname;
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

    pub fn register(&mut self, tunnel_id: u32, preferred_name: Option<String>) -> ResolvedHost {
        let hostname = self.assign_hostname(preferred_name);
        let host_id = self.assign_id();

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

    pub fn unregister_by_tunnel(&mut self, tunnel_id: u32) {
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
