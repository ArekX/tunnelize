use std::{
    io::{Error, ErrorKind},
    sync::Arc,
};

use log::{debug, info};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    common::{connection::ConnectionStream, request::DataRequest},
    connect_data_response,
    server::{configuration::ServerConfiguration, services::events::ServiceEvent, session},
    tunnel::configuration::ProxyConfiguration,
};

use super::super::services::Services;

use tokio::io::Result;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InitTunelRequest {
    pub endpoint_key: Option<String>,
    pub admin_key: Option<String>,
    pub proxies: Vec<ProxyConfiguration>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum InitTunnelResponse {
    Accepted { tunnel_id: Uuid },
    Rejected { reason: String },
}

connect_data_response!(InitTunelRequest, InitTunnelResponse);

pub async fn process_auth_tunnel(
    services: Arc<Services>,
    mut request: DataRequest<InitTunelRequest>,
) {
    let config = services.get_config();

    if let Err(e) = validate_server_access(&config, &mut request).await {
        debug!("Error validating server access: {:?}", e);
        return;
    }

    if let Err(e) = validate_requested_proxies(&mut request, &config).await {
        debug!("Error validating requested proxies: {:?}", e);
        return;
    }

    let has_admin_privileges = match resolve_admin_privileges(&mut request, &config).await {
        Ok(has_admin_privileges) => has_admin_privileges,
        Err(e) => {
            debug!("Error resolving admin privileges: {:?}", e);
            return;
        }
    };

    start_tunnel_session(services, has_admin_privileges, request.response_stream).await;
}

async fn validate_server_access(
    config: &ServerConfiguration,
    request: &mut DataRequest<InitTunelRequest>,
) -> Result<()> {
    if let Some(endpoint_key) = config.endpoint_key.as_ref() {
        if let Some(request_endpoint_key) = request.data.endpoint_key.as_ref() {
            if endpoint_key != request_endpoint_key {
                request
                    .response_stream
                    .respond_message(&InitTunnelResponse::Rejected {
                        reason: "Endpoint key is wrong or not valid".to_string(),
                    })
                    .await;

                return Err(Error::new(
                    ErrorKind::Other,
                    "Endpoint key is wrong or not valid",
                ));
            }
        }
    }

    Ok(())
}

async fn validate_requested_proxies(
    request: &mut DataRequest<InitTunelRequest>,
    config: &ServerConfiguration,
) -> Result<()> {
    // todo!()

    Ok(())
}

async fn resolve_admin_privileges(
    request: &mut DataRequest<InitTunelRequest>,
    config: &Arc<ServerConfiguration>,
) -> Result<bool> {
    if let Some(config_admin_key) = config.admin_key.as_ref() {
        if let Some(request_admin_key) = request.data.admin_key.as_ref() {
            if config_admin_key != request_admin_key {
                request
                    .response_stream
                    .respond_message(&InitTunnelResponse::Rejected {
                        reason: "Administration key is wrong or not valid".to_string(),
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

async fn start_tunnel_session(
    services: Arc<Services>,
    has_admin_privileges: bool,
    mut stream: ConnectionStream,
) {
    let (tunnel_session, channel_rx) = session::tunnel::create(has_admin_privileges);

    let tunnel_id = tunnel_session.get_id();

    info!("Tunnel connected. Assigned ID: {}", tunnel_id);

    if let Err(e) = stream
        .write_message(&InitTunnelResponse::Accepted { tunnel_id })
        .await
    {
        debug!("Error while sending tunnel accepted message: {:?}", e);
        return;
    }

    services
        .push_event(ServiceEvent::TunnelConnected {
            tunnel_session: tunnel_session.clone(),
        })
        .await;

    session::tunnel::start(services.clone(), tunnel_session, stream, channel_rx).await;

    services
        .push_event(ServiceEvent::TunnelDisconnected { tunnel_id })
        .await;
}
