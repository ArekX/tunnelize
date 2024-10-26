use std::collections::HashMap;

use uuid::Uuid;

use super::configuration::TcpEndpointConfig;

pub struct TunnelHost {
    max_port: u16,
    min_port: u16,
    host_tunnel_map: HashMap<u16, Tunnel>,
}

pub struct Tunnel {
    pub tunnel_id: Uuid,
    pub proxy_id: Uuid,
}

impl TunnelHost {
    pub fn new(config: &TcpEndpointConfig) -> Self {
        Self {
            host_tunnel_map: HashMap::new(),
            max_port: config.reserve_ports_to,
            min_port: config.reserve_ports_from,
        }
    }

    pub fn has_available_ports(&self) -> bool {
        self.host_tunnel_map.len() < (self.max_port - self.min_port) as usize
    }

    pub fn get_first_available_port(&self) -> Option<u16> {
        for port in self.min_port..self.max_port {
            if !self.host_tunnel_map.contains_key(&port) {
                return Some(port);
            }
        }

        None
    }

    pub fn resolve_port(&self, port: Option<u16>) -> Option<u16> {
        let port = match port {
            Some(port) => port,
            None => self.get_first_available_port()?,
        };

        if self.host_tunnel_map.contains_key(&port) || port < self.min_port || port > self.max_port
        {
            return self.get_first_available_port();
        }

        Some(port)
    }

    pub fn add_tunnel(
        &mut self,
        desired_port: Option<u16>,
        tunnel_id: Uuid,
        proxy_id: Uuid,
    ) -> Result<u16, String> {
        let Some(port) = self.resolve_port(desired_port) else {
            return Err("No available ports".to_string());
        };

        self.host_tunnel_map.insert(
            port,
            Tunnel {
                tunnel_id,
                proxy_id,
            },
        );

        Ok(port)
    }

    pub fn remove_tunnel(&mut self, tunnel_id: &Uuid) {
        self.host_tunnel_map
            .retain(|_, v| &v.tunnel_id != tunnel_id);
    }

    pub fn get_tunnel(&self, port: u16) -> Option<&Tunnel> {
        self.host_tunnel_map.get(&port)
    }
}
