use std::sync::Arc;

use tokio::io::Result;
use tokio::sync::{mpsc, oneshot};

use crate::tunnel::messages::TunnelMessageData;

use super::messages::HubChannelMessage;
use super::requests::{ServiceRequest, ServiceRequestData, ServiceResponse};
use super::services::Services;

pub async fn start(
    services: Arc<Services>,
    mut hub_receiver: mpsc::Receiver<HubChannelMessage>,
) -> Result<()> {
    loop {
        let response = match hub_receiver.recv().await {
            Some(response) => response,
            None => {
                break;
            }
        };

        match response {
            HubChannelMessage::Test(name) => {
                println!("Received name: {}", name);
            }
            HubChannelMessage::Tunnel(tunnel_mesage) => {
                if let Some(service) = services.get_service(tunnel_mesage.service_name.as_str()) {
                    let service_tx = service.get_service_tx();

                    let (response_tx, response_rx) = oneshot::channel::<ServiceResponse>();

                    match tunnel_mesage.data {
                        TunnelMessageData::Http(request) => {
                            service_tx
                                .send(ServiceRequest {
                                    data: ServiceRequestData::Http(request),
                                    response_tx,
                                })
                                .await
                                .unwrap();

                            let response = response_rx.await.unwrap();

                            match response {
                                ServiceResponse::Name(name) => {
                                    println!("Received name: {}", name);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
