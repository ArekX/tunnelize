use std::{
    collections::HashMap,
    io::{Error, ErrorKind},
    sync::Arc,
};

use log::{debug, info};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    common::connection::Connection,
    server::{
        configuration::ServerConfiguration,
        endpoints::messages::{
            RegisterTunnelRequest, RegisterTunnelResponse, ResolvedEndpointInfo,
        },
        services::events::ServiceEvent,
        session::{self, tunnel::TunnelProxyInfo},
    },
    tunnel::configuration::ProxyConfiguration,
};

use super::super::services::Services;

use tokio::io::Result;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InitTunelRequest {
    pub name: Option<String>,
    pub tunnel_key: Option<String>,
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
        endpoint_info: HashMap<Uuid, ResolvedEndpointInfo>,
    },
    Rejected {
        reason: String,
    },
}

pub async fn process(
    services: Arc<Services>,
    request: InitTunelRequest,
    mut response_stream: Connection,
) {
    let config = services.get_config();

    if let Err(e) = validate_server_access(&config, &request, &mut response_stream).await {
        debug!("Error validating server access: {:?}", e);
        return;
    }

    if let Err(e) = validate_requested_proxies(&request, &config, &mut response_stream).await {
        debug!("Error validating requested proxies: {:?}", e);
        return;
    }

    start_tunnel_session(services, request, response_stream).await;
}

async fn validate_server_access(
    config: &ServerConfiguration,
    request: &InitTunelRequest,
    response_stream: &mut Connection,
) -> Result<()> {
    if let Some(endpoint_key) = config.endpoint_key.as_ref() {
        if let Some(request_endpoint_key) = request.tunnel_key.as_ref() {
            if endpoint_key != request_endpoint_key {
                response_stream
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
    request: &InitTunelRequest,
    config: &ServerConfiguration,
    response_stream: &mut Connection,
) -> Result<()> {
    let mut errors: Vec<String> = vec![];

    for proxy in request.proxies.iter() {
        if let Some(endpoint) = config.endpoints.get(&proxy.endpoint_name) {
            if !endpoint.matches_proxy_type(&proxy.proxy) {
                errors.push(format!(
                    "Requested endpoint '{}' is of type '{}', but requested proxy is of type '{}'",
                    proxy.endpoint_name,
                    endpoint.get_type_string(),
                    proxy.proxy.get_type_string()
                ));
            }
        } else {
            errors.push(format!(
                "Requested non-existing endpoint: {}",
                proxy.endpoint_name
            ));
        }
    }

    if !errors.is_empty() {
        response_stream
            .respond_message(&InitTunnelResponse::Rejected {
                reason: format!("Proxy validation failed: {}", errors.join(", ")),
            })
            .await;
        return Err(Error::new(
            ErrorKind::Other,
            "Proxy validation failed.".to_owned(),
        ));
    }

    Ok(())
}

#[derive(Debug, Clone)]
pub struct ProxySession {
    pub proxy_id: Uuid,
    pub config: ProxyConfiguration,
}

async fn resolve_endpoint_info(
    tunnel_id: Uuid,
    request: &InitTunelRequest,
    services: &Arc<Services>,
) -> Result<(Vec<TunnelProxyInfo>, HashMap<Uuid, ResolvedEndpointInfo>)> {
    let mut service_proxies = HashMap::<String, Vec<ProxySession>>::new();
    let mut tunnel_proxy_info: Vec<TunnelProxyInfo> = Vec::new();

    for proxy in request.proxies.iter() {
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

    let mut proxy_data = HashMap::<Uuid, ResolvedEndpointInfo>::new();

    for (service_name, proxies) in service_proxies.iter() {
        let Ok(response) = services
            .get_endpoint_manager()
            .await
            .send_request(
                service_name,
                RegisterTunnelRequest {
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

        let RegisterTunnelResponse::Accepted { proxy_info } = response else {
            debug!("Endpoint '{}' rejected tunnel registration", service_name);
            return Err(Error::new(
                ErrorKind::Other,
                format!("Endpoint '{}' rejected tunnel registration", service_name),
            ));
        };

        for (proxy_id, endpoint_info) in proxy_info {
            tunnel_proxy_info.push(TunnelProxyInfo {
                details: endpoint_info.clone(),
                endpoint: service_name.clone(),
            });

            proxy_data.insert(proxy_id, endpoint_info);
        }
    }

    Ok((tunnel_proxy_info, proxy_data))
}

async fn start_tunnel_session(
    services: Arc<Services>,
    request: InitTunelRequest,
    mut response_stream: Connection,
) {
    let tunnel_id = Uuid::new_v4();

    let Ok((proxies, endpoint_info)) = resolve_endpoint_info(tunnel_id, &request, &services).await
    else {
        debug!("Error while resolving endpoint info!");
        return;
    };

    let (tunnel_session, channel_rx) =
        session::tunnel::create(tunnel_id.clone(), request.name.clone(), proxies);

    let tunnel_id = tunnel_session.get_id();

    info!("Tunnel connected. Assigned ID: {}", tunnel_id);

    if let Err(e) = response_stream
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
        })
        .await;

    info!("Tunnel session started: {}", tunnel_id);
    session::tunnel::start(
        services.clone(),
        tunnel_session,
        response_stream,
        channel_rx,
    )
    .await;

    info!("Tunnel session ended: {}", tunnel_id);
    services
        .push_event(ServiceEvent::TunnelDisconnected { tunnel_id })
        .await;
}
