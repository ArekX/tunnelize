use uuid::Uuid;

use crate::server::{incoming_requests::InputProxy, session::tunnel::TunnelSession};

#[derive(Clone, Debug)]
pub enum ServiceEvent {
    TunnelConnected {
        tunnel_session: TunnelSession,
        input_proxies: Vec<InputProxy>,
    },
    TunnelDisconnected {
        tunnel_id: Uuid,
    },
}
