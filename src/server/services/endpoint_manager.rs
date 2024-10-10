use super::{events::ServiceEvent, HandleServiceEvent};

pub struct EndpointManager {}

impl EndpointManager {
    pub fn new() -> Self {
        Self {}
    }
}

impl HandleServiceEvent for EndpointManager {
    fn handle_event(&mut self, event: ServiceEvent) {
        // TODO: implement
    }
}
