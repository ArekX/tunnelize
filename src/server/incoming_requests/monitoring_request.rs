use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::io::{Error, ErrorKind, Result};
use uuid::Uuid;

use crate::{
    common::connection::ConnectionStream,
    server::{configuration::ServerConfiguration, services::Services},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessMonitoringRequest {
    pub request: String,
    pub admin_key: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessMonitoringResponse {
    pub proxy_id: Uuid,
}

// TODO: Move for monitoring commands check
async fn resolve_admin_privileges(
    request: &ProcessMonitoringRequest,
    config: &Arc<ServerConfiguration>,
    response_stream: &mut ConnectionStream,
) -> Result<bool> {
    if let Some(config_admin_key) = config.admin_key.as_ref() {
        if let Some(request_admin_key) = request.admin_key.as_ref() {
            if config_admin_key != request_admin_key {
                response_stream
                    .respond_message(&ProcessMonitoringResponse {
                        proxy_id: Uuid::new_v4(),
                    })
                    .await;
                return Err(Error::new(
                    ErrorKind::Other,
                    "Administration key is wrong or not valid",
                ));
            }

            return Ok(true);
        }

        return Ok(false);
    }

    Ok(true)
}

pub async fn process(
    services: Arc<Services>,
    request: ProcessMonitoringRequest,
    mut response_stream: ConnectionStream,
) {
    let config = services.get_config();
}
