use log::{error, info};
use tokio::{
    io::Result,
    sync::mpsc::{channel, Sender}, task::JoinHandle,
};

use crate::{
    configuration::{ServerConfiguration, ServiceType},
    http::start_http_server,
    hub::{messages::HubMessage, requests::ServiceRequest, start_hub_server, Service},
};

pub async fn start_server(server_config: ServerConfiguration) -> Result<()> {
    let mut services = Vec::new();

    let (hub_tx, hub_rx) = channel::<HubMessage>(100);

    let mut hub_services: Vec<Service> = Vec::new();

    for (name, server) in server_config.services {
        let (service, handle) = start_service(name, server, hub_tx.clone())?;

        services.push(handle);
        hub_services.push(service);
    }

    services.push(tokio::spawn(async move {
        start_hub_server(hub_rx, hub_services).await;
    }));

    info!("Tunnelize servers initialized and running.");

    let mut has_error = false;

    for service in services {
        if let Err(e) = service.await {
            error!("Error procesing tunnel server: {}", e);
            has_error = true;
        }
    }

    if has_error {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "One or more servers failed.",
        ));
    }

    Ok(())
}

type ServiceHandle = JoinHandle<()>;

fn start_service(
    service_name: String,
    config: ServiceType,
    hub_tx: Sender<HubMessage>,
) -> Result<(Service, ServiceHandle)> {
    let (service_tx, mut service_rx) = channel::<ServiceRequest>(100);

    let handle: ServiceHandle = match config {
        ServiceType::Http(config) => {
                tokio::spawn(async move {
                    // start_http_server(server_config.hub_server_port, config).await
                })
        }
        _ => {
            info!("Unsupported server type, skipping.");
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Unsupported server type.",
            ));
        }
    }

    Ok((Service {
        name: service_name,
        service_tx: service_tx.clone(),
    }, handle))
}
