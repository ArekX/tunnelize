use std::collections::HashMap;

use tokio::net::TcpStream;

use super::host_list::ResolvedHost;

pub struct RequestedProxy {
    pub resolved_host: ResolvedHost,
    pub forward_address: String,
}

pub struct Tunnel {
    pub id: u32,
    pub proxy_map: HashMap<String, RequestedProxy>,
    pub stream: TcpStream,
}

pub struct TunnelList {
    id_counter: u32,
    tunnel_map: HashMap<u32, Tunnel>,
}

impl TunnelList {
    pub fn new() -> Self {
        TunnelList {
            id_counter: 0,
            tunnel_map: HashMap::new(),
        }
    }

    pub fn issue_tunnel_id(&mut self) -> u32 {
        let issued_id = self.id_counter;
        self.id_counter = self.id_counter.wrapping_add(1);
        issued_id
    }

    pub fn register(
        &mut self,
        tunnel_id: u32,
        stream: TcpStream,
        requested_proxies: Vec<RequestedProxy>,
    ) {
        let mut proxy_map = HashMap::new();

        for proxy in requested_proxies {
            proxy_map.insert(proxy.resolved_host.hostname.clone(), proxy);
        }

        self.tunnel_map.insert(
            tunnel_id,
            Tunnel {
                id: tunnel_id,
                proxy_map,
                stream,
            },
        );
    }

    pub fn is_registered(&self, id: u32) -> bool {
        self.tunnel_map.contains_key(&id)
    }

    pub fn remove_tunnel(&mut self, id: u32) {
        self.tunnel_map.remove(&id);
    }

    pub fn get_by_id(&mut self, id: u32) -> Option<&mut Tunnel> {
        self.tunnel_map.get_mut(&id)
    }
}
