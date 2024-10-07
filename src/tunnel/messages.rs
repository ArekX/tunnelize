pub enum ProxyMessage {
    LinkRequest {
        tunnel_id: String,
        session_key: String,
    },
}

pub enum TunnelRequestMessage {
    RequestLinkSession {
        service_name: String,
        session_key: String,
    },
}

pub enum TunnelResponseMessage {
    LinkSessionAccepted,
    LinkSessionRejected { reason: String },
}
