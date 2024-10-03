use std::{collections::HashMap, sync::Arc};

use tokio::sync::mpsc::{self, Sender};

use crate::configuration::ServiceDefinition;

use super::{messages::HubChannelMessage, requests::ServiceRequest, HubConfiguration};

pub struct HubService {
    definition: ServiceDefinition,
    service_tx: mpsc::Sender<ServiceRequest>,
}

impl HubService {
    pub fn new(service_tx: mpsc::Sender<ServiceRequest>, definition: ServiceDefinition) -> Self {
        Self {
            definition,
            service_tx,
        }
    }

    pub fn get_service_tx(&self) -> mpsc::Sender<ServiceRequest> {
        self.service_tx.clone()
    }
}

pub struct Services {
    services: HashMap<String, HubService>,
    hub_tx: Sender<HubChannelMessage>,
    config: Arc<HubConfiguration>,
}

impl Services {
    pub fn create(
        services: HashMap<String, HubService>,
        config: HubConfiguration,
        hub_tx: mpsc::Sender<HubChannelMessage>,
    ) -> Arc<Self> {
        Arc::new(Self {
            services,
            hub_tx,
            config: Arc::new(config),
        })
    }

    pub fn get_service(&self, name: &str) -> Option<&HubService> {
        self.services.get(name)
    }

    pub fn get_hub_tx(&self) -> mpsc::Sender<HubChannelMessage> {
        self.hub_tx.clone()
    }

    pub fn get_config(&self) -> Arc<HubConfiguration> {
        self.config.clone()
    }
}
