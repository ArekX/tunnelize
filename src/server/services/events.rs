use clap::builder::Str;
use uuid::Uuid;

use crate::{server::session::tunnel::TunnelSession, tunnel::configuration::TunnelProxy};

#[derive(Clone, Debug)]
pub enum ServiceEvent {
    TunnelConnected {
        tunnel_session: TunnelSession,
        tunnel_proxies: Vec<TunnelProxy>,
    },
    TunnelDisconnected {
        tunnel_id: Uuid,
    },
}
