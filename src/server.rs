use std::collections::HashMap;

use log::{error, info};
use tokio::{
    io::Result,
    sync::mpsc::{channel, Sender},
    task::JoinHandle,
};

use crate::{
    configuration::{ServerConfiguration, ServiceDefinition},
    http::start_http_service,
    hub::{messages::HubChannelMessage, requests::ServiceRequest, start_hub_server, HubService},
};

pub async fn start_server(server_config: ServerConfiguration) -> Result<()> {
    let mut services = Vec::new();

    let (hub_tx, hub_rx) = channel::<HubChannelMessage>(100);

    let mut hub_services: HashMap<String, HubService> = HashMap::new();

    for (name, server_def) in server_config.services {
        let (service, handle) = start_service(server_def, hub_tx.clone())?;

        services.push(handle);
        hub_services.insert(name, service);
    }

    let hub_config = server_config.hub.clone();

    services.push(tokio::spawn(async move {
        start_hub_server(hub_tx, hub_rx, hub_services, hub_config).await
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

type ServiceHandle = JoinHandle<Result<()>>;

fn start_service(
    service_def: ServiceDefinition,
    hub_tx: Sender<HubChannelMessage>,
) -> Result<(HubService, ServiceHandle)> {
    let (service_tx, service_rx) = channel::<ServiceRequest>(100);

    let hub_tx = hub_tx.clone();

    let hub_service = HubService::new(service_tx.clone(), service_def.clone());

    let handle: ServiceHandle = match service_def {
        ServiceDefinition::Http(config) => {
            tokio::spawn(async move { start_http_service(config, service_rx, hub_tx).await })
        }
        _ => {
            info!("Unsupported server type, skipping.");
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Unsupported server type.",
            ));
        }
    };

    Ok((hub_service, handle))
}
