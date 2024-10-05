use std::{collections::HashMap, sync::Arc};

use tokio::sync::mpsc;

use crate::{
    configuration::{TunnelConfiguration, TunnelDefinition},
    server::hub::{messages::HubChannelMessage, requests::ServiceRequest},
};

pub struct TunnelService {
    definition: TunnelDefinition,
    service_tx: mpsc::Sender<ServiceRequest>,
}

pub struct Services {
    services: HashMap<String, TunnelService>,
    hub_tx: mpsc::Sender<HubChannelMessage>,
    config: Arc<TunnelConfiguration>,
}

impl Services {
    pub fn create(
        services: HashMap<String, TunnelService>,
        config: TunnelConfiguration,
        hub_tx: mpsc::Sender<HubChannelMessage>,
    ) -> Arc<Self> {
        Arc::new(Self {
            services,
            hub_tx,
            config: Arc::new(config),
        })
    }

    pub fn get_service(&self, name: &str) -> Option<&TunnelService> {
        self.services.get(name)
    }

    pub fn get_hub_tx(&self) -> mpsc::Sender<HubChannelMessage> {
        self.hub_tx.clone()
    }

    pub fn get_config(&self) -> Arc<TunnelConfiguration> {
        self.config.clone()
    }
}
