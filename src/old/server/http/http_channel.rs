use std::sync::Arc;

use log::debug;
use tokio::sync::mpsc::Receiver;

use crate::server::hub::requests::{ServiceRequest, ServiceRequestData, ServiceResponse};

use super::services::Services;

pub async fn start(services: Arc<Services>, mut service_rx: Receiver<ServiceRequest>) {
    loop {
        let request = match service_rx.recv().await {
            Some(request) => request,
            None => {
                break;
            }
        };

        match request.data {
            ServiceRequestData::Http(_) => {
                if let Err(_) = request
                    .response_tx
                    .send(ServiceResponse::Name("Works!".to_string()))
                {
                    debug!("Failed to send response.");
                }
            }
            _ => {
                if let Err(_) = request
                    .response_tx
                    .send(ServiceResponse::Name("Unknown".to_string()))
                {
                    debug!("Failed to send response.");
                }
            }
        }
    }
}
