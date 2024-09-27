use std::collections::HashMap;

use tokio::net::TcpStream;
use uuid::Uuid;

use super::{host_list::ResolvedHost, ClientAuthorizeUser};

pub struct RequestedProxy {
    pub resolved_host: ResolvedHost,
}

pub struct Tunnel {
    pub stream: TcpStream,
    pub client_authorization: Option<ClientAuthorizeUser>,
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
        client_authorization: Option<ClientAuthorizeUser>,
    ) {
        let mut proxy_map = HashMap::new();

        for proxy in requested_proxies {
            proxy_map.insert(proxy.resolved_host.hostname.clone(), proxy);
        }

        self.tunnel_map.insert(
            tunnel_id,
            Tunnel {
                stream,
                client_authorization,
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
