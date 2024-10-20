use std::{io::{Error, ErrorKind}, sync::Arc, time::Duration};

use log::error;
use tokio::time::timeout;
use tokio::io::Result;
use uuid::Uuid;

use crate::{common::connection::ConnectionStream, server::{services::{Client, Services}, session::messages::{ClientLinkRequest, ClientLinkResponse}}};

use super::{protocol::{HttpRequestReader, HttpResponseBuilder}, tunnel_host::TunnelHost, HttpEndpointConfig};


pub async fn handle(
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
                .close_with_data(&HttpResponseBuilder::from_unauthorized(
                    &user.realm, 
                    "Access to the requested endpoint is not authorized. Please provide valid credentials.",
                ).build_bytes())
                .await;

            
            return false;
        }
    }

    true
}