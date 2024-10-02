use tokio::sync::oneshot;

use crate::http::messages::HttpTunnelMessage;

pub struct ServiceRequest {
    pub response_tx: oneshot::Sender<ServiceResponse>,
    pub data: ServiceRequestData,
}
pub enum ServiceRequestData {
    GetName,
    Http(HttpTunnelMessage)
}

pub enum ServiceResponse {
    Name(String),
}
