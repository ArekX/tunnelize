use std::{io::{Error, ErrorKind}, sync::Arc};

use log::error;
use tokio::io::Result;
use uuid::Uuid;

use crate::{common::connection::Connection, server::{services::{Client, Services}, session::messages::{ClientLinkRequest, ClientLinkResponse}}};

use super::{configuration::HttpEndpointConfig, protocol::{HttpRequestReader, HttpResponseBuilder}, tunnel_host::TunnelHost};


pub async fn handle(
    mut stream: Connection,
    tunnel_host: &TunnelHost,
    name: &str,
    config: &HttpEndpointConfig,
    services: &Arc<Services>,
) -> Result<()> {
    

    let request = match HttpRequestReader::new(&mut stream, config.max_client_input_wait_secs).await {
        Ok(request) => request,
        Err(e) => {
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
                    &HttpResponseBuilder::as_missing_header().build_bytes(),
                )
                .await;
            return Err(Error::new(ErrorKind::Other, "Host header is missing"));
        }
    };

    let Some(session) = tunnel_host.get_session(&hostname) else {
        stream
            .close_with_data(
                &HttpResponseBuilder::as_error(
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
        stream,
        None,
        Some(request.get_request_bytes()),
    );

    if let Err((error, link)) = services.get_client_manager().await.subscribe_client(client) {
        error!("Failed to subscribe client: {}", error);

        if let Some(mut link) = link {
            link.stream.close_with_data(
                &HttpResponseBuilder::as_error(&format!("Could not accept client. Reason: {}", error)).build_bytes(),
            )
            .await;
        }

        return Err(error);
    }

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
                        &Some(HttpResponseBuilder::as_error(&reason).build_bytes()),
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
                    &Some(HttpResponseBuilder::as_error("Failed to link client to tunnel")
                        .build_bytes()),
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
    stream: &mut Connection,
    config: &HttpEndpointConfig,
    request: &HttpRequestReader,
) -> bool {
    if let Some(user) = &config.require_authorization {
        if !request.is_authorization_matching(&user.username, &user.password) {
            stream
                .close_with_data(&HttpResponseBuilder::as_unauthorized(
                    &user.realm, 
                    "Access to the requested endpoint is not authorized. Please provide valid credentials.",
                ).build_bytes())
                .await;

            
            return false;
        }
    }

    true
}