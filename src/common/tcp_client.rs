use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use tokio::io::{self};
use tokio::{io::Result, net::TcpStream};

use super::connection::Connection;
use super::encryption::ClientTlsEncryption;
use super::protocol_socket::connect_to_address;
use super::validate::{Validatable, Validation};
use super::validate_rules::FileMustExist;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ClientEncryption {
    Tls {
        #[serde(skip_serializing_if = "Option::is_none", default)]
        ca_path: Option<String>,
    },
    None,
}

impl From<Option<ClientEncryption>> for ClientEncryption {
    fn from(encryption: Option<ClientEncryption>) -> Self {
        match encryption {
            Some(encryption) => encryption,
            None => ClientEncryption::None,
        }
    }
}

impl Validatable for ClientEncryption {
    fn validate(&self, result: &mut Validation) {
        if let Self::Tls { ca_path } = self {
            if let Some(cert) = ca_path {
                result.validate_rule::<FileMustExist>("ca_path", &cert);
            }
        }
    }
}

pub async fn create_tcp_client(
    address: &str,
    port: u16,
    encryption: ClientEncryption,
) -> Result<Connection> {
    match connect_to_address::<TcpStream>(address, port, ()).await {
        Ok((stream, _)) => match encryption {
            ClientEncryption::Tls { ca_path } => {
                let tls = ClientTlsEncryption::new(ca_path).await;

                info!("Connected to (TLS) server at {}", address);

                return Ok(tls.connect(stream, address).await?);
            }
            ClientEncryption::None => {
                info!("Connected to server at {}", address);
                return Ok(Connection::from(stream));
            }
        },
        Err(e) if e.kind() == io::ErrorKind::ConnectionRefused => {
            error!("Connection refused by server at {}", address);
            return Err(e);
        }
        Err(e) => {
            debug!("Error connecting to server: {:?}", e);
            return Err(e);
        }
    }
}
