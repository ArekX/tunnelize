use std::sync::Arc;

use init_link_session::process_init_link;

use crate::{common::connection::ConnectionStream, create_data_enum};

use super::services::Services;

mod init_link_session;

pub use init_link_session::{InitLinkRequest, InitLinkResponse};

create_data_enum!(TunnelRequestMessage, {
    InitLinkRequest -> InitLinkResponse
});

pub async fn handle(
    services: &Arc<Services>,
    stream: &mut ConnectionStream,
    message: TunnelRequestMessage,
) {
    match message {
        TunnelRequestMessage::InitLinkRequest(request) => {
            process_init_link(services, request, stream).await;
        }
    }
}
