use std::collections::HashMap;

use log::error;
use serde::Serialize;

use crate::{
    common::channel::{create_channel, DataResponse, RequestReceiver, RequestSender},
    server::{
        configuration::EndpointConfiguration,
        endpoints::messages::{
            EndpointChannelRequest, EndpointChannelResponse, RemoveTunnelRequest,
        },
    },
};

use super::{events::ServiceEvent, HandleServiceEvent};

#[derive(Clone)]
pub struct Endpoint {
    pub name: String,
    pub definition: EndpointConfiguration,
    channel_tx: RequestSender<EndpointChannelRequest>,
}

impl Into<EndpointInfo> for &Endpoint {
    fn into(self) -> EndpointInfo {
        EndpointInfo {
            name: self.name.clone(),
            definition: self.definition.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct EndpointInfo {
    pub name: String,
    pub definition: EndpointConfiguration,
}

impl Endpoint {
    pub fn new(
        name: String,
        definition: EndpointConfiguration,
        channel_tx: RequestSender<EndpointChannelRequest>,
    ) -> Self {
        Self {
            name: name,
            definition: definition,
            channel_tx: channel_tx,
        }
    }

    pub fn get_channel_tx(&self) -> RequestSender<EndpointChannelRequest> {
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

    pub fn add_endpoint(
        &mut self,
        service_name: &str,
        config: &EndpointConfiguration,
    ) -> RequestReceiver<EndpointChannelRequest> {
        let (channel_tx, channel_rx) = create_channel::<EndpointChannelRequest>();

        let endpoint = Endpoint::new(service_name.to_owned(), config.clone(), channel_tx);

        self.endpoints.insert(endpoint.name.clone(), endpoint);

        channel_rx
    }

    fn get_endpoint_channel_tx(
        &self,
        service_name: &str,
    ) -> Option<RequestSender<EndpointChannelRequest>> {
        match self.endpoints.get(service_name) {
            Some(endpoint) => Some(endpoint.get_channel_tx()),
            None => None,
        }
    }

    pub async fn send_request<T: Into<EndpointChannelRequest> + DataResponse>(
        &self,
        service_name: &str,
        request: T,
    ) -> tokio::io::Result<T::Response>
    where
        T::Response: TryFrom<EndpointChannelResponse>,
    {
        let Some(tunnel_tx) = self.get_endpoint_channel_tx(service_name) else {
            return Err(tokio::io::Error::new(
                tokio::io::ErrorKind::NotFound,
                format!("Endpoint not found: {:?}", service_name),
            ));
        };

        tunnel_tx.request(request).await
    }

    pub fn get_count(&self) -> usize {
        self.endpoints.len()
    }

    pub fn list_endpoints(&self) -> Vec<EndpointInfo> {
        self.endpoints.values().map(|e| e.into()).collect()
    }

    pub fn get_endpoint_info(&self, service_name: &str) -> Option<EndpointInfo> {
        self.endpoints.get(service_name).map(|e| e.into())
    }
}

impl HandleServiceEvent for EndpointManager {
    async fn handle_event(&mut self, event: &ServiceEvent) {
        match event {
            ServiceEvent::TunnelDisconnected { tunnel_id } => {
                for endpoint_name in self.endpoints.keys() {
                    if let Err(e) = self
                        .send_request(
                            &endpoint_name,
                            RemoveTunnelRequest {
                                tunnel_id: tunnel_id.clone(),
                            },
                        )
                        .await
                    {
                        error!(
                            "Error while sending RemoveTunnelRequest to endpoint '{}': {:?}",
                            endpoint_name, e
                        );
                    }
                }
            }
            _ => {}
        }
    }
}
