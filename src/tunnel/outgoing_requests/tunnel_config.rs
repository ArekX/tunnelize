use crate::{
    server::incoming_requests::{ConfigRequest, ProcessConfigRequest, ProcessConfigResponse},
    tunnel::configuration::TunnelConfiguration,
};

pub async fn get_tunnel_config(
    config: TunnelConfiguration,
) -> tokio::io::Result<ProcessConfigResponse> {
    config
        .create_tcp_client()
        .await?
        .request_message(ProcessConfigRequest {
            tunnel_key: config.tunnel_key.clone(),
            request: ConfigRequest::GetPublicEndpointConfig,
        })
        .await
}
