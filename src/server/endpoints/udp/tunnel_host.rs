use std::collections::HashMap;

use uuid::Uuid;

use super::configuration::UdpEndpointConfig;

#[derive(Clone, Debug)]
pub struct Tunnel {
    pub tunnel_id: Uuid,
    pub proxy_id: Uuid,
}

pub struct TunnelHost {
    max_port: u16,
    min_port: u16,
    allow_desired_port: bool,
    host_tunnel_map: HashMap<u16, Tunnel>,
}

impl TunnelHost {
    pub fn new(config: &UdpEndpointConfig) -> Self {
        Self {
            host_tunnel_map: HashMap::new(),
            max_port: config.reserve_ports_to,
            min_port: config.reserve_ports_from,
            allow_desired_port: config.get_allow_desired_port(),
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
            Some(port) => {
                if self.allow_desired_port {
                    port
                } else {
                    self.get_first_available_port()?
                }
            }
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

    pub fn get_tunnel(&self, port: u16) -> Option<Tunnel> {
        match self.host_tunnel_map.get(&port) {
            Some(tunnel) => Some(tunnel.clone()),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn get_test_config() -> UdpEndpointConfig {
        UdpEndpointConfig {
            reserve_ports_from: 1000,
            reserve_ports_to: 1010,
            address: None,
            allow_desired_port: None,
            inactivity_timeout: None,
            full_hostname_template: None,
        }
    }

    fn get_test_tunnel_host() -> TunnelHost {
        TunnelHost::new(&get_test_config())
    }

    #[test]
    fn test_has_available_ports() {
        let tunnel_host = get_test_tunnel_host();
        assert!(tunnel_host.has_available_ports());
    }

    #[test]
    fn test_get_first_available_port() {
        let tunnel_host = get_test_tunnel_host();
        assert_eq!(tunnel_host.get_first_available_port(), Some(1000));
    }

    #[test]
    fn test_resolve_port() {
        let mut tunnel_host = get_test_tunnel_host();
        assert_eq!(tunnel_host.resolve_port(Some(1001)), Some(1001));
        assert_eq!(tunnel_host.resolve_port(None), Some(1000));
        tunnel_host
            .add_tunnel(Some(1000), Uuid::new_v4(), Uuid::new_v4())
            .unwrap();
        assert_eq!(tunnel_host.resolve_port(Some(1000)), Some(1001));
    }

    #[test]
    fn test_add_tunnel() {
        let mut tunnel_host = get_test_tunnel_host();
        let tunnel_id = Uuid::new_v4();
        let proxy_id = Uuid::new_v4();
        let port = tunnel_host
            .add_tunnel(Some(1000), tunnel_id, proxy_id)
            .unwrap();
        assert_eq!(port, 1000);
        assert!(tunnel_host.get_tunnel(1000).is_some());
    }

    #[test]
    fn test_remove_tunnel() {
        let mut tunnel_host = get_test_tunnel_host();
        let tunnel_id = Uuid::new_v4();
        let proxy_id = Uuid::new_v4();
        tunnel_host
            .add_tunnel(Some(1000), tunnel_id, proxy_id)
            .unwrap();
        tunnel_host.remove_tunnel(&tunnel_id);
        assert!(tunnel_host.get_tunnel(1000).is_none());
    }

    #[test]
    fn test_get_tunnel() {
        let mut tunnel_host = get_test_tunnel_host();
        let tunnel_id = Uuid::new_v4();
        let proxy_id = Uuid::new_v4();
        tunnel_host
            .add_tunnel(Some(1000), tunnel_id, proxy_id)
            .unwrap();
        let tunnel = tunnel_host.get_tunnel(1000).unwrap();
        assert_eq!(tunnel.tunnel_id, tunnel_id);
        assert_eq!(tunnel.proxy_id, proxy_id);
    }
}
