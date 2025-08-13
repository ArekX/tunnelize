use uuid::Uuid;

pub struct TunnelData {
    pub tunnel_id: Option<Uuid>,
    pub failed_heartbeats: u16,
}

impl Default for TunnelData {
    fn default() -> Self {
        Self::new()
    }
}

impl TunnelData {
    pub fn new() -> Self {
        Self {
            tunnel_id: None,
            failed_heartbeats: 0,
        }
    }

    pub fn set_tunnel_id(&mut self, tunnel_id: Uuid) {
        if self.tunnel_id.is_some() {
            panic!("Tunnel ID already set");
        }

        self.tunnel_id = Some(tunnel_id);
    }

    pub fn get_tunnel_id(&self) -> Option<Uuid> {
        self.tunnel_id
    }

    pub fn record_success_heartbeat(&mut self) {
        self.failed_heartbeats = 0;
    }

    pub fn record_failed_heartbeat(&mut self) {
        self.failed_heartbeats += 1;
    }

    pub fn too_many_failed_heartbeats(&self) -> bool {
        self.failed_heartbeats >= 5
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn create_tunnel_data_with_id() -> TunnelData {
        let mut tunnel_data = TunnelData::new();
        tunnel_data.set_tunnel_id(Uuid::new_v4());
        tunnel_data
    }

    #[test]
    fn test_new() {
        let tunnel_data = TunnelData::new();
        assert!(tunnel_data.tunnel_id.is_none());
    }

    #[test]
    fn test_set_tunnel_id() {
        let mut tunnel_data = TunnelData::new();
        let tunnel_id = Uuid::new_v4();
        tunnel_data.set_tunnel_id(tunnel_id);
        assert_eq!(tunnel_data.tunnel_id, Some(tunnel_id));
    }

    #[test]
    #[should_panic(expected = "Tunnel ID already set")]
    fn test_set_tunnel_id_twice() {
        let mut tunnel_data = create_tunnel_data_with_id();
        tunnel_data.set_tunnel_id(Uuid::new_v4());
    }

    #[test]
    fn test_get_tunnel_id() {
        let tunnel_data = create_tunnel_data_with_id();
        assert!(tunnel_data.get_tunnel_id().is_some());
    }

    #[test]
    fn test_get_tunnel_id_none() {
        let tunnel_data = TunnelData::new();
        assert!(tunnel_data.get_tunnel_id().is_none());
    }
}
