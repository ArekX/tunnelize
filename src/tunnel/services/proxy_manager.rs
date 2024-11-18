use std::collections::HashMap;

use log::error;
use tokio::net::TcpStream;
use tokio::{io::Result, net::UdpSocket};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::common::connection::ConnectionStreamContext;
use crate::common::data_bridge::UdpSession;
use crate::common::protocol_socket::{connect_to_address, UdpSocketConnectionContext};
use crate::tunnel::configuration::ProxyConfiguration;
use crate::{common::connection::Connection, tunnel::configuration::TunnelProxy};

pub struct Proxy {
    pub address: String,
    pub port: u16,
    pub endpoint_name: String,
    pub protocol: ProxyProtocol,
}

impl Proxy {
    pub async fn create_forward_connection(
        &self,
    ) -> Result<(Connection, Option<ConnectionStreamContext>)> {
        Ok(match self.protocol {
            ProxyProtocol::Tcp => {
                match connect_to_address::<TcpStream>(&self.address, self.port, ()).await {
                    Ok((stream, _)) => (Connection::from(stream), None),
                    Err(e) => {
                        error!("Failed to connect to forward address: {}", e);
                        return Err(e);
                    }
                }
            }
            ProxyProtocol::Udp { ref bind_address } => {
                match connect_to_address::<UdpSocket>(
                    &self.address,
                    self.port,
                    UdpSocketConnectionContext {
                        bind_address: bind_address.clone(),
                    },
                )
                .await
                {
                    Ok((socket, address)) => (
                        Connection::from(socket),
                        Some(ConnectionStreamContext::Udp(UdpSession {
                            address,
                            cancel_token: CancellationToken::new(),
                        })),
                    ),
                    Err(e) => {
                        error!("Failed to connect to forward address: {}", e);
                        return Err(e);
                    }
                }
            }
        })
    }
}

pub enum ProxyProtocol {
    Tcp,
    Udp { bind_address: Option<String> },
}

impl From<&ProxyConfiguration> for ProxyProtocol {
    fn from(value: &ProxyConfiguration) -> Self {
        match value {
            ProxyConfiguration::Http { .. } => ProxyProtocol::Tcp,
            ProxyConfiguration::Tcp { .. } => ProxyProtocol::Tcp,
            ProxyConfiguration::Udp { bind_address, .. } => ProxyProtocol::Udp {
                bind_address: bind_address.clone(),
            },
        }
    }
}

pub struct ProxyManager {
    proxy_map: HashMap<Uuid, Proxy>,
}

impl ProxyManager {
    pub fn new() -> Self {
        Self {
            proxy_map: HashMap::new(),
        }
    }

    pub fn get_forward_address(&self, id: &Uuid) -> Option<(String, u16)> {
        self.proxy_map
            .get(id)
            .map(|session| (session.address.clone(), session.port))
    }

    pub fn get_proxy(&self, id: &Uuid) -> Option<&Proxy> {
        self.proxy_map.get(id)
    }

    pub fn add_proxy(&mut self, proxy: &TunnelProxy) -> Uuid {
        let id = Uuid::new_v4();

        let proxy = Proxy {
            address: proxy.address.clone(),
            port: proxy.port,
            endpoint_name: proxy.endpoint_name.clone(),
            protocol: ProxyProtocol::from(&proxy.endpoint_config),
        };

        self.proxy_map.insert(id, proxy);

        id
    }

    pub async fn create_forward_connection(
        &self,
        id: &Uuid,
    ) -> Result<(Connection, Option<ConnectionStreamContext>)> {
        if let Some(session) = self.proxy_map.get(id) {
            return session.create_forward_connection().await;
        }

        error!("Failed to find proxy session with id: {}", id);

        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Failed to find proxy session",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tunnel::configuration::{ProxyConfiguration, TunnelProxy};

    fn create_test_proxy_configuration() -> TunnelProxy {
        TunnelProxy {
            address: "127.0.0.1".to_string(),
            port: 8080,
            endpoint_name: "test".to_string(),
            endpoint_config: ProxyConfiguration::Tcp { desired_port: None },
        }
    }

    #[test]
    fn test_add_proxy() {
        let mut manager = ProxyManager::new();
        let proxy_config = create_test_proxy_configuration();
        let id = manager.add_proxy(&proxy_config);

        assert!(manager.get_proxy(&id).is_some());
    }

    #[test]
    fn test_get_forward_address() {
        let mut manager = ProxyManager::new();
        let proxy_config = create_test_proxy_configuration();
        let id = manager.add_proxy(&proxy_config);

        let address = manager.get_forward_address(&id);
        assert!(address.is_some());
        assert_eq!(address.unwrap(), ("127.0.0.1".to_string(), 8080));
    }

    #[test]
    fn test_get_proxy() {
        let mut manager = ProxyManager::new();
        let proxy_config = create_test_proxy_configuration();
        let id = manager.add_proxy(&proxy_config);

        let proxy = manager.get_proxy(&id);
        assert!(proxy.is_some());
        assert_eq!(proxy.unwrap().address, "127.0.0.1");
    }
}
