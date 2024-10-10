use log::info;
use tokio::io::{self, Result};
use uuid::Uuid;

use crate::common::connection::ConnectionStream;
use crate::server::incoming_requests::{AuthLinkRequest, AuthLinkResponse, ServerRequestMessage};

pub async fn authenticate_link(
    tunnel_id: Uuid,
    session_id: Uuid,
    server: &mut ConnectionStream,
) -> Result<()> {
    let auth_response: AuthLinkResponse = server
        .request_message(&ServerRequestMessage::AuthLink(AuthLinkRequest {
            tunnel_id,
            session_id,
        }))
        .await?;

    match auth_response {
        AuthLinkResponse::Accepted => {
            info!("Tunnel session accepted: {}", tunnel_id);
        }
        AuthLinkResponse::Rejected { reason } => {
            return Err(io::Error::new(io::ErrorKind::Other, reason));
        }
    }

    Ok(())
}
