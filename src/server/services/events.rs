use uuid::Uuid;

use crate::server::session::tunnel::TunnelSession;

#[derive(Clone, Debug)]
pub enum ServiceEvent {
    TunnelConnected { tunnel_session: TunnelSession },
    TunnelDisconnected { tunnel_id: Uuid },
    LinkDisconnected { client_id: Uuid, session_id: Uuid },
    LinkRejected { client_id: Uuid, session_id: Uuid },
}
