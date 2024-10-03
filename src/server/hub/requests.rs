use tokio::sync::oneshot;

use super::super::http::messages::HttpTunnelMessage;

pub struct ServiceRequest {
    pub response_tx: oneshot::Sender<ServiceResponse>,
    pub data: ServiceRequestData,
}
pub enum ServiceRequestData {
    GetName,
    Http(HttpTunnelMessage),
}

pub enum ServiceResponse {
    Name(String),
}
