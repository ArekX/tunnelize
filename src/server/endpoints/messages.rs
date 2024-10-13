use uuid::Uuid;

use crate::tunnel::configuration::ProxyConfiguration;

#[derive(Clone, Debug)]
pub enum EndpointMessage {
    TunnelConnected {
        tunnel_id: Uuid,
        proxy_configuration: ProxyConfiguration,
    },
    TunnelDisconnected {
        tunnel_id: Uuid,
    },
}
