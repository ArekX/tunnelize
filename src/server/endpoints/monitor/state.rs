use std::sync::Arc;

use crate::server::services::Services;

use super::configuration::MonitorEndpointConfig;

#[derive(Clone)]
pub struct AppState {
    pub services: Arc<Services>,
    pub config: Arc<MonitorEndpointConfig>,
    pub name: String,
}

impl AppState {
    pub fn new(services: Arc<Services>, config: Arc<MonitorEndpointConfig>, name: String) -> Self {
        Self {
            services,
            config,
            name,
        }
    }
}
