use std::collections::HashMap;

use uuid::Uuid;

use crate::common::text::get_random_letters;

use super::HttpEndpointConfig;

pub struct TunnelHost {
    hostname_template: String,
    allow_custom_hostnames: bool,
    host_tunnel_map: HashMap<String, HostTunnelSession>,
}

pub struct HostTunnelSession {
    pub tunnel_id: Uuid,
    pub proxy_id: Uuid,
}

impl TunnelHost {
    pub fn new(config: &HttpEndpointConfig) -> Self {
        Self {
            host_tunnel_map: HashMap::new(),
            allow_custom_hostnames: config.get_allow_custom_hostnames(),
            hostname_template: config.hostname_template.clone(),
        }
    }

    fn generate_new_name(&self, desired_hostname: &Option<String>, append_suffix: bool) -> String {
        if self.allow_custom_hostnames {
            if append_suffix {
                return format!(
                    "{}-{}",
                    desired_hostname
                        .clone()
                        .unwrap_or_else(|| get_random_letters(5)),
                    get_random_letters(5)
                );
            }

            return desired_hostname
                .clone()
                .unwrap_or_else(|| get_random_letters(5));
        };

        get_random_letters(5)
    }

    fn generate_unique_hostname(&self, desired_hostname: &Option<String>) -> String {
        let mut hostname = self
            .hostname_template
            .replace("{name}", &self.generate_new_name(desired_hostname, false));

        while self.host_tunnel_map.contains_key(&hostname) {
            hostname = self
                .hostname_template
                .replace("{name}", &self.generate_new_name(desired_hostname, true));
        }

        hostname
    }

    pub fn register_host(
        &mut self,
        desired_hostname: &Option<String>,
        tunnel_id: &Uuid,
        proxy_id: &Uuid,
    ) -> String {
        let hostname = self.generate_unique_hostname(desired_hostname);

        self.host_tunnel_map.insert(
            hostname.clone(),
            HostTunnelSession {
                tunnel_id: *tunnel_id,
                proxy_id: *proxy_id,
            },
        );

        hostname
    }

    pub fn remove_tunnel_by_id(&mut self, tunnel_id: &Uuid) {
        self.host_tunnel_map
            .retain(|_, v| &v.tunnel_id != tunnel_id);
    }

    pub fn get_session(&self, hostname: &str) -> Option<&HostTunnelSession> {
        self.host_tunnel_map.get(hostname)
    }
}
