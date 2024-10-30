use log::{debug, error, info};
use tokio::io::{self};
use tokio::{io::Result, net::TcpStream};

use super::connection::Connection;
use super::encryption::{ClientEncryptionType, ClientTlsEncryption};
use super::protocol_socket::connect_to_address;

pub async fn create_tcp_client(
    address: &str,
    port: u16,
    encryption: Option<ClientEncryptionType>,
) -> Result<Connection> {
    match connect_to_address::<TcpStream>(address, port).await {
        Ok((stream, _)) => match encryption {
            Some(encryption_type) => {
                let tls = ClientTlsEncryption::new(encryption_type).await;

                info!("Connected to (TLS) server at {}", address);

                return Ok(tls.connect(stream, address).await?);
            }
            None => {
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
