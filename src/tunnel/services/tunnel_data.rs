use uuid::Uuid;

pub struct TunnelData {
    pub tunnel_id: Option<Uuid>,
}

impl TunnelData {
    pub fn new() -> Self {
        Self { tunnel_id: None }
    }

    pub fn set_tunnel_id(&mut self, tunnel_id: Uuid) {
        if self.tunnel_id.is_some() {
            panic!("Tunnel ID already set");
        }

        self.tunnel_id = Some(tunnel_id);
    }

    pub fn get_tunnel_id(&self) -> Option<Uuid> {
        self.tunnel_id.clone()
    }
}
