use std::sync::Arc;

use crate::server::services::Services;

pub fn has_tunnel_access(services: &Arc<Services>, key: Option<&String>) -> bool {
    let config = services.get_config();
    if let Some(endpoint_key) = config.tunnel_key.as_ref() {
        if let Some(request_key) = key {
            return endpoint_key == request_key;
        }
        return false;
    }

    true
}

pub fn has_monitoring_access(services: &Arc<Services>, key: Option<&String>) -> bool {
    let config = services.get_config();
    if let Some(endpoint_key) = config.monitor_key.as_ref() {
        if let Some(request_key) = key {
            return endpoint_key == request_key;
        }
        return false;
    }

    true
}

#[cfg(test)]
mod tests {

    use tokio_util::sync::CancellationToken;

    use super::*;
    use crate::server::configuration::ServerConfiguration;
    use crate::server::services::Services;
    use std::collections::HashMap;
    use std::sync::Arc;

    fn create_services(tunnel_key: Option<String>, monitor_key: Option<String>) -> Arc<Services> {
        let config = ServerConfiguration {
            tunnel_key,
            monitor_key,
            server_port: None,
            server_address: None,
            max_tunnel_input_wait: None,
            endpoints: HashMap::new(),
            encryption: None,
            max_tunnels: None,
            max_clients: None,
            max_proxies_per_tunnel: None,
        };

        Arc::new(Services::new(config, CancellationToken::new()))
    }

    #[test]
    fn test_has_tunnel_access() {
        let services = create_services(Some("test".to_string()), None);

        assert!(has_tunnel_access(&services, Some(&"test".to_string())));
        assert!(!has_tunnel_access(&services, Some(&"test2".to_string())));
        assert!(!has_tunnel_access(&services, None));
    }

    #[test]
    fn test_has_tunnel_access_no_key() {
        let services = create_services(None, None);

        assert!(has_tunnel_access(&services, Some(&"test".to_string())));
        assert!(has_tunnel_access(&services, None));
    }

    #[test]
    fn test_has_monitoring_access() {
        let services = create_services(None, Some("test".to_string()));

        assert!(has_monitoring_access(&services, Some(&"test".to_string())));
        assert!(!has_monitoring_access(
            &services,
            Some(&"test2".to_string())
        ));
        assert!(!has_monitoring_access(&services, None));
    }

    #[test]
    fn test_has_monitoring_access_no_key() {
        let services = create_services(None, None);

        assert!(has_monitoring_access(&services, Some(&"test".to_string())));
        assert!(has_monitoring_access(&services, None));
    }
}
