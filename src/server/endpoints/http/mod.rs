mod configuration;
mod protocol;
mod tunnel_host;

use std::{
    io::{Error, ErrorKind},
    sync::Arc,
    time::Duration,
};

pub use configuration::HttpEndpointConfig;
use log::{debug, error, info};
use protocol::get_error_response;
use serde::{Deserialize, Serialize};
use tokio::{
    io::Result,
    net::TcpListener,
    sync::{
        mpsc::{self},
        oneshot,
    },
    time::timeout,
};
use tunnel_host::TunnelHost;
use uuid::Uuid;

use crate::{
    common::{
        channel_request::{send_channel_request, ChannelRequest},
        connection::ConnectionStream,
    },
    server::{
        endpoints::EndpointInfo,
        services::{Client, EndpointMessage, Services},
        session::messages::{ClientLinkRequest, ClientLinkResponse, TunnelSessionMessage},
    },
    tunnel::configuration::ProxyConfiguration,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum HttpEndpointInfo {
    AssignedUrl {
        proxy_id: Uuid,
        assigned_url: String,
    },
}

pub async fn start(
    services: Arc<Services>,
    name: String,
    config: HttpEndpointConfig,
    mut channel_rx: mpsc::Receiver<EndpointMessage>,
) -> Result<()> {
    let mut tunnel_host = TunnelHost::new();

    let listener = match TcpListener::bind(format!("0.0.0.0:{}", config.port)).await {
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
            message = channel_rx.recv() => {
                match message {
                    Some(message) => {
                        debug!("Received endpoint message '{:?}'", message);
                        if let Err(e) = handle_endpoint_message(message, &config, &mut tunnel_host, &services).await {
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
                .send_and_close(&get_error_response(
                    "",
                    "Failed to read request data within allowed time frame",
                ))
                .await;
            return Err(Error::new(ErrorKind::Other, e));
        }
    };

    let hostname = match protocol::find_host_header_value(&request) {
        Some(hostname) => hostname,
        None => {
            stream
                .send_and_close(&get_error_response(&request, "Host header is missing"))
                .await;
            return Err(Error::new(ErrorKind::Other, "Host header is missing"));
        }
    };

    let Some(tunnel_id) = tunnel_host.get_tunnel_id(&hostname) else {
        stream
            .send_and_close(&get_error_response(
                &request,
                "No tunnel is assigned for the requested hostname",
            ))
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
        Ok(_) => {
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
                    .send_and_close(&get_error_response(
                        &request,
                        "Failed to link client to tunnel",
                    ))
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
    message: EndpointMessage,
    config: &HttpEndpointConfig,
    tunnel_host: &mut TunnelHost,
    services: &Arc<Services>,
) -> Result<()> {
    match message {
        EndpointMessage::TunnelConnected {
            tunnel_id,
            proxy_configuration,
        } => {
            let ProxyConfiguration::Http(http_config) = proxy_configuration else {
                return Err(Error::new(
                    ErrorKind::Other,
                    "Invalid proxy configuration for HTTP endpoint",
                ));
            };

            let name = http_config.desired_name.unwrap_or("random".to_string());

            let hostname = config.host_template.replace("{name}", &name);

            let full_url = config.full_url_template.replace("{hostname}", &hostname);

            info!(
                "Tunnel ID '{}' connected to endpoint '{}' with hostname '{}'",
                tunnel_id, name, hostname
            );

            tunnel_host.add_tunnel(hostname, tunnel_id);

            send_assigned_url(&tunnel_id, &full_url, services).await?;
        }
        EndpointMessage::TunnelDisconnected { tunnel_id } => {
            tunnel_host.remove_tunnel_by_id(&tunnel_id);
        }
    }

    Ok(())
}

async fn send_assigned_url(
    tunnel_id: &Uuid,
    full_url: &str,
    services: &Arc<Services>,
) -> Result<()> {
    let Some(session_tx) = services
        .get_tunnel_manager()
        .await
        .get_session_tx(tunnel_id)
    else {
        return Err(Error::new(
            ErrorKind::Other,
            "Failed to get tunnel session for tunnel ID",
        ));
    };

    if let Err(e) = session_tx
        .send(TunnelSessionMessage::EndpointInfo(EndpointInfo::Http(
            HttpEndpointInfo::AssignedUrl {
                proxy_id: Uuid::new_v4(),
                assigned_url: full_url.to_string(),
            },
        )))
        .await
    {
        return Err(Error::new(
            ErrorKind::Other,
            format!("Failed to send assigned URL to tunnel session: {}", e),
        ));
    }

    Ok(())
}
