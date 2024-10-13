use std::collections::HashMap;

use log::error;
use tokio::sync::mpsc::Sender;

use crate::server::{configuration::EndpointConfiguration, endpoints::messages::EndpointMessage};

use super::{events::ServiceEvent, HandleServiceEvent};

#[derive(Clone, Debug)]
pub struct Endpoint {
    pub name: String,
    pub definition: EndpointConfiguration,
    channel_tx: Sender<EndpointMessage>,
}

impl Endpoint {
    pub fn new(
        name: String,
        definition: EndpointConfiguration,
        channel_tx: Sender<EndpointMessage>,
    ) -> Self {
        Self {
            name: name,
            definition: definition,
            channel_tx: channel_tx,
        }
    }

    pub fn get_channel_tx(&self) -> Sender<EndpointMessage> {
        self.channel_tx.clone()
    }
}

pub struct EndpointManager {
    endpoints: HashMap<String, Endpoint>,
}

impl EndpointManager {
    pub fn new() -> Self {
        Self {
            endpoints: HashMap::new(),
        }
    }

    pub fn add_endpoint(&mut self, endpoint: Endpoint) {
        self.endpoints.insert(endpoint.name.clone(), endpoint);
    }
}

impl HandleServiceEvent for EndpointManager {
    async fn handle_event(&mut self, event: &ServiceEvent) {
        match event {
            ServiceEvent::TunnelConnected {
                tunnel_session,
                tunnel_proxies,
            } => {
                let tunnel_id = tunnel_session.get_id();
                for tunnel_proxy in tunnel_proxies.iter() {
                    if let Some(endpoint) = self.endpoints.get(&tunnel_proxy.endpoint_name) {
                        if let Err(e) = endpoint
                            .get_channel_tx()
                            .send(EndpointMessage::TunnelConnected {
                                tunnel_id,
                                proxy_configuration: tunnel_proxy.proxy.clone(),
                            })
                            .await
                        {
                            error!(
                                "Error sending tunnel connected message to endpoint '{}': {:?}",
                                tunnel_proxy.endpoint_name, e
                            );
                        }
                    }
                }
            }
            ServiceEvent::TunnelDisconnected { tunnel_id } => {
                for endpoint in self.endpoints.values() {
                    if let Err(e) = endpoint
                        .get_channel_tx()
                        .send(EndpointMessage::TunnelDisconnected {
                            tunnel_id: tunnel_id.clone(),
                        })
                        .await
                    {
                        error!(
                            "Error sending tunnel disconnected message to endpoint '{}': {:?}",
                            endpoint.name, e
                        );
                    }
                }
            }
        }
    }
}
