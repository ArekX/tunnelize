use super::{configuration::TcpEndpointConfig, tunnel_host::TunnelHost};
use crate::{common::configuration::ServerEncryption, server::services::Services as MainServices};
use log::error;
use std::sync::Arc;
use tokio::sync::{Mutex, MutexGuard};
use tokio_util::sync::CancellationToken;

pub struct TcpServices {
    config: Arc<TcpEndpointConfig>,
    name: String,
    tunnel_host: Arc<Mutex<TunnelHost>>,
    cancel_token: CancellationToken,
    server_encryption: ServerEncryption,
    main_services: Arc<MainServices>,
}

impl TcpServices {
    pub fn new(
        config: TcpEndpointConfig,
        name: String,
        main_services: Arc<MainServices>,
    ) -> tokio::io::Result<Self> {
        let cancel_token = main_services.get_cancel_token();
        let tunnel_host = Arc::new(Mutex::new(TunnelHost::new(&config)));

        let server_encryption = match config.encryption.to_encryption(&main_services.get_config()) {
            Ok(encryption) => encryption,
            Err(e) => {
                error!("Failed to get server encryption: {}", e);
                return Err(e);
            }
        };

        Ok(Self {
            config: Arc::new(config),
            tunnel_host,
            name,
            server_encryption,
            cancel_token,
            main_services,
        })
    }

    pub fn get_server_encryption(&self) -> ServerEncryption {
        self.server_encryption.clone()
    }

    pub fn get_endpoint_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_config(&self) -> Arc<TcpEndpointConfig> {
        self.config.clone()
    }

    pub async fn get_tunnel_host(&self) -> MutexGuard<TunnelHost> {
        self.tunnel_host.lock().await
    }

    pub fn get_cancel_token(&self) -> CancellationToken {
        self.cancel_token.clone()
    }

    pub fn get_main_services(&self) -> Arc<MainServices> {
        self.main_services.clone()
    }
}
