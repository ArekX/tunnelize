use crate::{
    server::incoming_requests::{ConfigRequest, ProcessConfigRequest, ProcessConfigResponse},
    tunnel::{client::create_server_connection, configuration::TunnelConfiguration},
};

pub async fn get_tunnel_config(
    config: TunnelConfiguration,
) -> tokio::io::Result<ProcessConfigResponse> {
    create_server_connection(&config)
        .await?
        .request_message(ProcessConfigRequest {
            tunnel_key: config.tunnel_key.clone(),
            request: ConfigRequest::GetPublicEndpointConfig,
        })
        .await
}
