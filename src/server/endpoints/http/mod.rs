mod configuration;
mod protocol;
mod tunnel_host;

use std::{
    collections::HashMap,
    io::{Error, ErrorKind},
    sync::Arc,
    time::Duration,
};

pub use configuration::HttpEndpointConfig;
use log::{debug, error, info};
use protocol::get_error_response;
use rustls::crypto::hash::Hash;
use serde::{Deserialize, Serialize};
use tokio::{io::Result, net::TcpListener, time::timeout};
use tunnel_host::TunnelHost;
use uuid::Uuid;

use crate::{
    common::{
        channel::{OkResponse, Request, RequestReceiver},
        connection::ConnectionStream,
        text::get_random_letters,
    },
    server::{
        endpoints::messages::RegisterProxyResponse,
        services::{Client, Services},
        session::messages::ClientLinkRequest,
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
            request = channel_rx.recv() => {
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

    let request = match timeout(max_input_duration, protocol::read_http_request(&mut stream)).await
    {
        Ok(request) => request,
        Err(e) => {
            stream
                .write_and_shutdown(
                    &get_error_response(
                        "",
                        "Failed to read request data within allowed time frame",
                    )
                    .as_bytes(),
                )
                .await;
            return Err(Error::new(ErrorKind::Other, e));
        }
    };

    // TODO: Check authorization

    let hostname = match protocol::find_host_header_value(&request) {
        Some(hostname) => hostname,
        None => {
            stream
                .write_and_shutdown(
                    &get_error_response(&request, "Host header is missing").as_bytes(),
                )
                .await;
            return Err(Error::new(ErrorKind::Other, "Host header is missing"));
        }
    };

    let Some(tunnel_id) = tunnel_host.get_tunnel_id(&hostname) else {
        stream
            .write_and_shutdown(
                &get_error_response(&request, "No tunnel is assigned for the requested hostname")
                    .as_bytes(),
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
        Some(request.clone().into_bytes()),
    );

    services.get_client_manager().await.add_client(client);

    match services
        .get_tunnel_manager()
        .await
        .send_session_request(
            tunnel_id,
            ClientLinkRequest {
                client_id,
                endpoint_name: name.to_owned(),
            },
        )
        .await
    {
        Ok(result) => {
            println!(
                "Client ID '{}' linked to tunnel ID '{}'",
                client_id, tunnel_id
            );
        }
        Err(e) => {
            error!("Failed to link client to tunnel: {}", e);

            if let Some(mut client) = services.get_client_manager().await.take_client(client_id) {
                client
                    .stream
                    .write_and_shutdown(
                        &get_error_response(&request, "Failed to link client to tunnel").as_bytes(),
                    )
                    .await;
            }

            return Err(Error::new(
                ErrorKind::Other,
                "Failed to link client to tunnel",
            ));
        }
    }

    Ok(())
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
                let ProxyConfiguration::Http(http_config) = &proxy_session.config else {
                    debug!("Proxy session configuration passed is not for Http endpoint");
                    continue;
                };

                let hostname =
                    tunnel_host.register_host(&http_config.desired_name, &proxy_request.tunnel_id);

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
