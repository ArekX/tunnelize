mod configuration;
mod protocol;
mod tunnel_host;

use std::{
    collections::HashMap,
    io::{Error, ErrorKind},
    sync::Arc,
    time::Duration,
};

pub use configuration::{AuthorizeUser, HttpEndpointConfig};

use log::{debug, error, info};
use protocol::{HttpRequestReader, HttpResponseBuilder};
use serde::{Deserialize, Serialize};
use tokio::{io::Result, net::TcpListener, time::timeout};
use tunnel_host::TunnelHost;
use uuid::Uuid;

use crate::{
    common::{
        channel::{OkResponse, Request, RequestReceiver},
        connection::ConnectionStream,
    },
    server::{
        endpoints::messages::RegisterProxyResponse,
        services::{Client, Services},
        session::messages::{ClientLinkRequest, ClientLinkResponse},
    },
    tunnel::configuration::ProxyConfiguration,
};

use super::messages::{EndpointInfo, EndpointRequest, RemoveTunnelRequest};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HttpEndpointInfo {
    assigned_url: String,
}

pub async fn start(
    services: Arc<Services>,
    name: String,
    config: HttpEndpointConfig,
    mut channel_rx: RequestReceiver<EndpointRequest>,
) -> Result<()> {
    let mut tunnel_host = TunnelHost::new(&config);

    let listener = match TcpListener::bind(config.get_bind_address()).await {
        Ok(listener) => listener,
        Err(e) => {
            error!("Failed to bind client listener: {}", e);
            return Err(Error::new(
                ErrorKind::Other,
                "Failed to bind client listener",
            ));
        }
    };

    loop {
        tokio::select! {
            request = channel_rx.wait_for_requests() => {
                match request {
                    Some(request) => {
                        debug!("Received endpoint message");
                        if let Err(e) = handle_endpoint_message(request, &config,  &mut tunnel_host).await {
                            error!("Failed to handle endpoint message: {}", e);
                        }
                    },
                    None => {
                        info!("Endpoint '{}' channel has been shutdown", name);
                        return Ok(());
                    }
                }
            }
            client = listener.accept() => {
                match client {
                    Ok((stream, stream_address)) => {
                        info!("Accepted connection from client: {}", stream_address);
                        if let Err(e) = handle_client_request(ConnectionStream::from(stream), &tunnel_host, &name, &config, &services).await {
                            error!("Failed to handle client request: {}", e);
                        }
                    },
                    Err(e) => {
                        error!("Failed to accept client connection: {}", e);
                        continue;
                    }
                };
            }
        }
    }
}

async fn handle_client_request(
    mut stream: ConnectionStream,
    tunnel_host: &TunnelHost,
    name: &str,
    config: &HttpEndpointConfig,
    services: &Arc<Services>,
) -> Result<()> {
    let max_input_duration = Duration::from_secs(config.max_client_input_wait_secs);

    let request = match timeout(max_input_duration, HttpRequestReader::new(&mut stream)).await {
        Ok(request) => request,
        Err(e) => {
            stream
                .close_with_data(
                    &HttpResponseBuilder::from_error(
                        "Failed to read request data within allowed time frame",
                    )
                    .build_bytes(),
                )
                .await;
            return Err(Error::new(ErrorKind::Other, e));
        }
    };

    if !check_authorization(&mut stream, config, &request).await {
        return Err(Error::new(ErrorKind::Other, "Unauthorized"));
    }

    let hostname = match request.find_hostname() {
        Some(hostname) => hostname,
        None => {
            stream
                .close_with_data(
                    &HttpResponseBuilder::from_error("Host header is missing").build_bytes(),
                )
                .await;
            return Err(Error::new(ErrorKind::Other, "Host header is missing"));
        }
    };

    let Some(session) = tunnel_host.get_session(&hostname) else {
        stream
            .close_with_data(
                &HttpResponseBuilder::from_error(
                    "No tunnel is assigned for the requested hostname",
                )
                .build_bytes(),
            )
            .await;
        return Err(Error::new(
            ErrorKind::Other,
            "No tunnel is assigned for the requested hostname",
        ));
    };

    let client_id = Uuid::new_v4();

    let client = Client::new(
        client_id,
        name.to_owned(),
        hostname,
        stream,
        Some(request.get_request_bytes()),
    );

    services.get_client_manager().await.subscribe_client(client);

    match services
        .get_tunnel_manager()
        .await
        .send_session_request(
            &session.tunnel_id,
            ClientLinkRequest {
                client_id: client_id,
                proxy_id: session.proxy_id,
            },
        )
        .await
    {
        Ok(result) => {
            if let ClientLinkResponse::Rejected { reason } = result {
                error!(
                    "Client ID '{}' rejected by tunnel ID '{}': {}",
                    client_id, session.tunnel_id, reason
                );

                services
                    .get_client_manager()
                    .await
                    .cancel_client(
                        &client_id,
                        &HttpResponseBuilder::from_error(&reason).build_bytes(),
                    )
                    .await;

                return Err(Error::new(ErrorKind::Other, reason));
            }

            println!(
                "Client ID '{}' linked to tunnel ID '{}'",
                client_id, session.tunnel_id
            );
        }
        Err(e) => {
            error!("Failed to link client to tunnel: {}", e);

            services
                .get_client_manager()
                .await
                .cancel_client(
                    &client_id,
                    &HttpResponseBuilder::from_error("Failed to link client to tunnel")
                        .build_bytes(),
                )
                .await;

            return Err(Error::new(
                ErrorKind::Other,
                "Failed to link client to tunnel",
            ));
        }
    }

    Ok(())
}

async fn check_authorization(
    stream: &mut ConnectionStream,
    config: &HttpEndpointConfig,
    request: &HttpRequestReader,
) -> bool {
    if let Some(user) = &config.require_authorization {
        if !request.is_authorization_matching(&user.username, &user.password) {
            stream
                .close_with_data(&HttpResponseBuilder::from_unauthorized(&user.realm).build_bytes())
                .await;
            return false;
        }
    }

    true
}

async fn handle_endpoint_message(
    mut request: Request<EndpointRequest>,
    config: &HttpEndpointConfig,
    tunnel_host: &mut TunnelHost,
) -> Result<()> {
    match &request.data {
        EndpointRequest::RegisterProxyRequest(proxy_request) => {
            let mut proxy_info = HashMap::<Uuid, EndpointInfo>::new();

            for proxy_session in proxy_request.proxy_sessions.iter() {
                let ProxyConfiguration::Http { desired_name } = &proxy_session.config else {
                    debug!("Proxy session configuration passed is not for Http endpoint");
                    continue;
                };

                let hostname = tunnel_host.register_host(
                    &desired_name,
                    &proxy_request.tunnel_id,
                    &proxy_session.proxy_id,
                );

                info!(
                    "Tunnel ID '{}' connected to http endpoint with hostname '{}'",
                    proxy_request.tunnel_id, hostname
                );

                proxy_info.insert(
                    proxy_session.proxy_id,
                    EndpointInfo::Http(HttpEndpointInfo {
                        assigned_url: config.get_full_url(&hostname),
                    }),
                );
            }

            request.respond(RegisterProxyResponse { proxy_info }).await;
        }
        EndpointRequest::RemoveTunnelRequest(RemoveTunnelRequest { tunnel_id }) => {
            info!("Removing tunnel ID '{}' from http endpoint.", tunnel_id);
            tunnel_host.remove_tunnel_by_id(&tunnel_id);
            request.respond(OkResponse).await;
        }
    }

    Ok(())
}
