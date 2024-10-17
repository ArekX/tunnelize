use std::{
    collections::HashMap,
    io::{Error, ErrorKind},
    sync::Arc,
};

use log::{debug, info};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    common::data_request::DataRequest,
    connect_data_response,
    server::{
        configuration::ServerConfiguration,
        endpoints::messages::{EndpointInfo, RegisterProxyRequest},
        services::events::ServiceEvent,
        session::{self},
    },
    tunnel::configuration::{ProxyConfiguration, TunnelProxy},
};

use super::super::services::Services;

use tokio::io::Result;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InitTunelRequest {
    pub endpoint_key: Option<String>,
    pub admin_key: Option<String>,
    pub proxies: Vec<InputProxy>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InputProxy {
    pub proxy_id: Uuid,
    pub endpoint_name: String,
    pub proxy: ProxyConfiguration,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum InitTunnelResponse {
    Accepted {
        tunnel_id: Uuid,
        endpoint_info: HashMap<Uuid, EndpointInfo>,
    },
    Rejected {
        reason: String,
    },
}

connect_data_response!(InitTunelRequest -> InitTunnelResponse);

pub async fn process_init_tunnel(
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

    start_tunnel_session(services, has_admin_privileges, request).await;
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
    // TODO: Validate requested proxies
    // TODO: Assign proxy IDs here, this is to be used by endpoints for info, also send the IDs back to tunnel

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

#[derive(Debug, Clone)]
pub struct ProxySession {
    pub proxy_id: Uuid,
    pub config: ProxyConfiguration,
}

async fn resolve_endpoint_info(
    tunnel_id: Uuid,
    request: &DataRequest<InitTunelRequest>,
    services: &Arc<Services>,
) -> Result<HashMap<Uuid, EndpointInfo>> {
    let mut service_proxies = HashMap::<String, Vec<ProxySession>>::new();

    for proxy in request.data.proxies.iter() {
        let sessions = {
            if !service_proxies.contains_key(&proxy.endpoint_name) {
                service_proxies.insert(proxy.endpoint_name.clone(), Vec::new());
            }

            service_proxies.get_mut(&proxy.endpoint_name).unwrap()
        };

        let proxy_session = ProxySession {
            proxy_id: proxy.proxy_id,
            config: proxy.proxy.clone(),
        };

        sessions.push(proxy_session);
    }

    let mut proxy_data = HashMap::<Uuid, EndpointInfo>::new();

    for (service_name, proxies) in service_proxies.iter() {
        let Ok(response) = services
            .get_endpoint_manager()
            .await
            .send_request(
                service_name,
                RegisterProxyRequest {
                    tunnel_id,
                    proxy_sessions: proxies.clone(),
                },
            )
            .await
        else {
            debug!(
                "Error while sending RegisterProxyRequest to endpoint '{}'",
                service_name
            );
            return Err(Error::new(
                ErrorKind::Other,
                format!(
                    "Error while sending RegisterProxyRequest to endpoint '{}'",
                    service_name
                ),
            ));
        };

        for (proxy_id, endpoint_info) in response.proxy_info {
            proxy_data.insert(proxy_id, endpoint_info);
        }
    }

    Ok(proxy_data)
}

async fn start_tunnel_session(
    services: Arc<Services>,
    has_admin_privileges: bool,
    request: DataRequest<InitTunelRequest>,
) {
    let (tunnel_session, channel_rx) = session::tunnel::create(has_admin_privileges);

    let tunnel_id = tunnel_session.get_id();

    info!("Tunnel connected. Assigned ID: {}", tunnel_id);

    let Ok(endpoint_info) = resolve_endpoint_info(tunnel_id, &request, &services).await else {
        debug!("Error while resolving endpoint info!");
        return;
    };

    let mut stream = request.response_stream;

    if let Err(e) = stream
        .write_message(&InitTunnelResponse::Accepted {
            tunnel_id,
            endpoint_info,
        })
        .await
    {
        debug!("Error while sending tunnel accepted message: {:?}", e);
        return;
    }

    services
        .push_event(ServiceEvent::TunnelConnected {
            tunnel_session: tunnel_session.clone(),
            input_proxies: request.data.proxies.clone(),
        })
        .await;

    session::tunnel::start(services.clone(), tunnel_session, stream, channel_rx).await;

    services
        .push_event(ServiceEvent::TunnelDisconnected { tunnel_id })
        .await;
}
