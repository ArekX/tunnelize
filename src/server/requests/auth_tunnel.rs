use std::sync::Arc;

use log::info;
use serde::{Deserialize, Serialize};

use crate::{
    common::{connection::ConnectionStream, transport::respond_message},
    server::{configuration::ServerConfiguration, session},
    tunnel::configuration::ProxyConfiguration,
};

use super::{
    super::{messages::ServerResponseMessage, services::Services},
    ServerRequest,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AuthTunelRequest {
    pub endpoint_key: Option<String>,
    pub admin_key: Option<String>,
    pub proxies: Vec<ProxyConfiguration>,
}

pub async fn process_auth_tunel_request<'a>(
    services: Arc<Services>,
    mut request: ServerRequest<AuthTunelRequest>,
) {
    let config = services.get_config();

    if let Some(endpoint_key) = config.endpoint_key.as_ref() {
        if let Some(request_endpoint_key) = request.data.endpoint_key.as_ref() {
            if endpoint_key != request_endpoint_key {
                request
                    .stream
                    .respond_message(&ServerResponseMessage::AuthLinkRejected {
                        reason: "Endpoint key is wrong or not valid".to_string(),
                    })
                    .await;
            }
        }
    }

    let has_admin_privileges = match resolve_admin_privileges(
        &request.data.admin_key,
        &mut request.stream,
        &services.get_config(),
    )
    .await
    {
        Ok(has_admin_privileges) => has_admin_privileges,
        Err(_) => {
            return;
        }
    };

    start_tunnel_session(services, has_admin_privileges, request.stream).await;
}

async fn resolve_admin_privileges<'a>(
    admin_key: &Option<String>,
    stream: &mut ConnectionStream,
    config: &Arc<ServerConfiguration>,
) -> Result<bool, ()> {
    if let Some(config_admin_key) = config.admin_key.as_ref() {
        if let Some(request_admin_key) = admin_key.as_ref() {
            if config_admin_key != request_admin_key {
                stream
                    .respond_message(&ServerResponseMessage::AuthLinkRejected {
                        reason: "Administration key is wrong or not valid".to_string(),
                    })
                    .await;
                return Err(());
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
    let mut tunnel_manager = services.get_tunnel_manager().await;

    let (tunnel, channel_rx) = session::tunnel::create(has_admin_privileges);

    let tunnel_id = tunnel.get_id();

    info!("Tunnel connected. Assigned ID: {}", tunnel_id);

    tunnel_manager.register_tunnel_session(tunnel);

    stream
        .respond_message(&ServerResponseMessage::AuthTunnelAccepted { tunnel_id })
        .await;

    session::tunnel::start(services.clone(), stream, channel_rx).await;
}
