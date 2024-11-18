use std::collections::HashMap;

use log::error;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Serialize, Deserialize, Clone)]
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
                            "Error while sending RemoveTunnelRequest to endpoint '{}': {}",
                            endpoint_name, e
                        );
                    }
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::configuration::EndpointConfiguration;
    use crate::server::endpoints::http::configuration::HttpEndpointConfig;

    fn create_test_endpoint_manager() -> EndpointManager {
        EndpointManager::new()
    }

    fn create_test_endpoint_config() -> EndpointConfiguration {
        EndpointConfiguration::Http(HttpEndpointConfig {
            port: 8080,
            encryption: None,
            address: None,
            max_client_input_wait_secs: None,
            hostname_template: "host-{name}.example.com".to_string(),
            full_url_template: None,
            allow_custom_hostnames: None,
            require_authorization: None,
        })
    }

    #[test]
    fn test_add_endpoint() {
        let mut manager = create_test_endpoint_manager();
        let config = create_test_endpoint_config();
        let _ = manager.add_endpoint("test_service", &config);
        assert!(manager.get_endpoint_info("test_service").is_some());
    }

    #[test]
    fn test_get_count() {
        let mut manager = create_test_endpoint_manager();
        let config = create_test_endpoint_config();
        manager.add_endpoint("test_service", &config);
        assert_eq!(manager.get_count(), 1);
    }

    #[test]
    fn test_list_endpoints() {
        let mut manager = create_test_endpoint_manager();
        let config = create_test_endpoint_config();
        manager.add_endpoint("test_service", &config);
        let endpoints = manager.list_endpoints();
        assert_eq!(endpoints.len(), 1);
        assert_eq!(endpoints[0].name, "test_service");
    }

    #[test]
    fn test_get_endpoint_info() {
        let mut manager = create_test_endpoint_manager();
        let config = create_test_endpoint_config();
        manager.add_endpoint("test_service", &config);
        let endpoint_info = manager.get_endpoint_info("test_service");
        assert!(endpoint_info.is_some());
        assert_eq!(endpoint_info.unwrap().name, "test_service");
    }
}
