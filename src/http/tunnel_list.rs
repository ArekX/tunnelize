use std::collections::HashMap;

use tokio::net::TcpStream;
use uuid::Uuid;

use super::host_list::ResolvedHost;

pub struct RequestedProxy {
    pub resolved_host: ResolvedHost,
    pub forward_address: String,
}

pub struct Tunnel {
    pub id: Uuid,
    pub proxy_map: HashMap<String, RequestedProxy>,
    pub stream: TcpStream,
}

pub struct TunnelList {
    tunnel_map: HashMap<Uuid, Tunnel>,
}

impl TunnelList {
    pub fn new() -> Self {
        TunnelList {
            tunnel_map: HashMap::new(),
        }
    }

    pub fn issue_tunnel_id(&self) -> Uuid {
        Uuid::new_v4()
    }

    pub fn register(
        &mut self,
        tunnel_id: Uuid,
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

    pub fn is_registered(&self, id: Uuid) -> bool {
        self.tunnel_map.contains_key(&id)
    }

    pub fn remove_tunnel(&mut self, id: Uuid) {
        self.tunnel_map.remove(&id);
    }

    pub fn get_by_id(&mut self, id: Uuid) -> Option<&mut Tunnel> {
        self.tunnel_map.get_mut(&id)
    }
}
