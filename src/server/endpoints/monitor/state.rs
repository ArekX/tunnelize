use std::sync::Arc;

use crate::server::services::Services;

#[derive(Clone)]
pub struct AppState {
    pub services: Arc<Services>,
}

impl AppState {
    pub fn new(services: Arc<Services>) -> Self {
        Self { services }
    }
}
