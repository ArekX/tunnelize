use std::collections::HashMap;

use uuid::Uuid;

use crate::common::text::get_random_letters;

use super::HttpEndpointConfig;

pub struct TunnelHost {
    hostname_template: String,
    host_tunnel_map: HashMap<String, Uuid>,
}

impl TunnelHost {
    pub fn new(config: &HttpEndpointConfig) -> Self {
        Self {
            host_tunnel_map: HashMap::new(),
            hostname_template: config.hostname_template.clone(),
        }
    }

    pub fn register_host(&mut self, desired_hostname: &Option<String>, tunnel_id: &Uuid) -> String {
        let name = desired_hostname
            .clone()
            .unwrap_or_else(|| get_random_letters(5));

        let hostname = self.hostname_template.replace("{name}", &name);

        self.host_tunnel_map.insert(hostname.clone(), *tunnel_id);

        hostname
    }

    pub fn remove_tunnel_by_id(&mut self, tunnel_id: &Uuid) {
        self.host_tunnel_map.retain(|_, v| v != tunnel_id);
    }

    pub fn get_tunnel_id(&self, hostname: &str) -> Option<&Uuid> {
        self.host_tunnel_map.get(hostname)
    }
}
