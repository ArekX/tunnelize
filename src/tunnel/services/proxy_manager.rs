use std::collections::HashMap;

use log::error;
use tokio::net::TcpStream;
use tokio::{io::Result, net::UdpSocket};
use uuid::Uuid;

use crate::tunnel::configuration::ProxyConfiguration;
use crate::{common::connection::ConnectionStream, tunnel::configuration::TunnelProxy};

pub struct ProxySession {
    pub proxy_id: Uuid,
    pub endpoint_name: String,
    pub forward_address: String,
    pub protocol: ProxyProtocol,
}

impl ProxySession {
    pub fn new(
        proxy_id: Uuid,
        endpoint_name: String,
        forward_address: String,
        protocol: ProxyProtocol,
    ) -> Self {
        Self {
            proxy_id,
            endpoint_name,
            forward_address,
            protocol,
        }
    }

    pub async fn create_forward_connection(&self) -> Result<ConnectionStream> {
        Ok(match self.protocol {
            ProxyProtocol::Tcp => match TcpStream::connect(self.forward_address.clone()).await {
                Ok(stream) => ConnectionStream::from(stream),
                Err(e) => {
                    error!("Failed to connect to forward address: {}", e);
                    return Err(e);
                }
            },
            ProxyProtocol::Udp => match UdpSocket::bind(self.forward_address.clone()).await {
                Ok(socket) => ConnectionStream::from(socket),
                Err(e) => {
                    error!("Failed to bind to forward address: {}", e);
                    return Err(e);
                }
            },
        })
    }
}

pub enum ProxyProtocol {
    Tcp,
    Udp,
}

impl From<&ProxyConfiguration> for ProxyProtocol {
    fn from(value: &ProxyConfiguration) -> Self {
        match value {
            ProxyConfiguration::Http(_) => ProxyProtocol::Tcp,
            ProxyConfiguration::Tcp { .. } => ProxyProtocol::Tcp,
            ProxyConfiguration::Udp { .. } => ProxyProtocol::Udp,
        }
    }
}

pub struct ProxyManager {
    proxy_session_map: HashMap<Uuid, ProxySession>,
}

impl ProxyManager {
    pub fn new() -> Self {
        Self {
            proxy_session_map: HashMap::new(),
        }
    }

    pub fn get_forward_address(&self, id: &Uuid) -> Option<String> {
        self.proxy_session_map
            .get(id)
            .map(|session| session.forward_address.clone())
    }

    pub fn add_proxy(&mut self, proxy: &TunnelProxy) -> Uuid {
        let id = Uuid::new_v4();

        let proxy_session = ProxySession {
            proxy_id: id,
            endpoint_name: proxy.endpoint_name.clone(),
            forward_address: proxy.forward_address.clone(),
            protocol: ProxyProtocol::from(&proxy.config),
        };

        self.proxy_session_map.insert(id, proxy_session);

        println!("Added proxy session: {:?}", id);

        id
    }

    pub async fn create_forward_connection(&self, id: &Uuid) -> Result<ConnectionStream> {
        if let Some(session) = self.proxy_session_map.get(id) {
            return session.create_forward_connection().await;
        }

        error!("Failed to find proxy session with id: {}", id);

        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Failed to find proxy session",
        ))
    }
}
