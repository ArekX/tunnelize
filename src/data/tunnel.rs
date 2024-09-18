use std::{collections::HashMap, sync::Arc};

use tokio::{net::TcpStream, sync::Mutex};

use crate::messages::ResolvedLink;

pub struct ProxyLink {
    pub forward_address: String,
    pub client_address: String,
}

pub struct TunnelLink {
    pub tunnel_id: u32,
    pub link_id: u32,
}

pub struct Tunnel {
    pub id: u32,
    pub proxy_map: HashMap<u32, ProxyLink>,
    pub stream: TcpStream,
}

pub struct TunnelList {
    tunnel_map: HashMap<u32, Tunnel>,
    client_address_map: HashMap<String, TunnelLink>,
}

impl TunnelList {
    pub fn new() -> Self {
        TunnelList {
            tunnel_map: HashMap::new(),
            client_address_map: HashMap::new(),
        }
    }

    pub fn register(
        &mut self,
        tunnel_id: u32,
        tunnel_stream: TcpStream,
        resolved_links: &Vec<ResolvedLink>,
    ) {
        let mut tunnel = Tunnel {
            id: tunnel_id,
            proxy_map: HashMap::new(),
            stream: tunnel_stream,
        };

        let mut link_counter: u32 = 0;

        for link in resolved_links {
            link_counter = link_counter.wrapping_add(1);
            tunnel.proxy_map.insert(
                link_counter,
                ProxyLink {
                    forward_address: link.forward_address.clone(), // will be read in monitoring
                    client_address: link.client_address.clone(),
                },
            );
            self.client_address_map.insert(
                link.client_address.clone(),
                TunnelLink {
                    tunnel_id,
                    link_id: link.link_id,
                },
            );
        }

        self.tunnel_map.insert(tunnel_id, tunnel);
    }

    pub fn is_registered(&self, id: u32) -> bool {
        self.tunnel_map.contains_key(&id)
    }

    pub fn remove_tunnel(&mut self, id: u32) {
        if let Some(tunnel) = self.tunnel_map.get(&id) {
            for link in tunnel.proxy_map.values() {
                println!("Removing link: {}", link.client_address);
                self.client_address_map.remove(&link.client_address);
            }
            self.tunnel_map.remove(&id);
        }
    }

    pub fn find_by_client_address(
        &mut self,
        client_address: &String,
    ) -> Option<(u32, &mut Tunnel)> {
        if let Some(tunnel) = self.client_address_map.get(client_address) {
            return Some((
                tunnel.link_id,
                self.tunnel_map.get_mut(&tunnel.tunnel_id).unwrap(),
            ));
        }

        None
    }
}

pub type MainTunnelList = Arc<Mutex<TunnelList>>;

pub fn create_tunnel_list() -> MainTunnelList {
    Arc::new(Mutex::new(TunnelList::new()))
}
