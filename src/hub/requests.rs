use tokio::sync::oneshot;

pub struct ServiceRequest {
    pub response_tx: oneshot::Sender<ServiceResponse>,
    pub data: ServiceRequestData,
}
pub enum ServiceRequestData {
    GetName,
}

pub enum ServiceResponse {
    Name(String),
}
