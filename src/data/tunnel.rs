use std::sync::Arc;

use tokio::{net::TcpStream, sync::Mutex};

pub struct Tunnel {
    pub id: u32,
    pub hostname: String,
    pub stream: TcpStream,
}

pub struct TunnelList {
    tunnels: Vec<Tunnel>,
}

impl TunnelList {
    pub fn new() -> Self {
        TunnelList {
            tunnels: Vec::new(),
        }
    }

    pub fn register(&mut self, tunnel: Tunnel) {
        self.tunnels.push(tunnel);
    }

    pub fn is_registered(&self, id: u32) -> bool {
        self.tunnels.iter().any(|t| t.id == id)
    }

    pub fn remove_tunnel(&mut self, id: u32) {
        self.tunnels.retain(|t| t.id != id);
    }

    pub fn find_by_hostname(&mut self, hostname: String) -> Option<&mut Tunnel> {
        // create hashmap hostname -> tunnel_id
        // upon registering a tunnel, add the hostname to the hashmap
        self.tunnels
            .iter_mut()
            .find(|tunnel| tunnel.hostname == hostname)
    }
}

pub type MainTunnelList = Arc<Mutex<TunnelList>>;

pub fn create_tunnel_list() -> MainTunnelList {
    Arc::new(Mutex::new(TunnelList::new()))
}
