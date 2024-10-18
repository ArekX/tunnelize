use log::info;
use tokio::io::{self, Result};
use uuid::Uuid;

use crate::common::connection::ConnectionStream;
use crate::server::incoming_requests::{InitLinkRequest, InitLinkResponse};

pub async fn authenticate_link(
    tunnel_id: Uuid,
    session_id: Uuid,
    server: &mut ConnectionStream,
) -> Result<()> {
    let auth_response: InitLinkResponse = server
        .request_message(InitLinkRequest {
            tunnel_id,
            session_id,
        })
        .await?;

    match auth_response {
        InitLinkResponse::Accepted => {
            info!("Tunnel session accepted: {}", tunnel_id);
        }
        InitLinkResponse::Rejected { reason } => {
            return Err(io::Error::new(io::ErrorKind::Other, reason));
        }
    }

    Ok(())
}
