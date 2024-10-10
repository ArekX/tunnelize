use std::sync::Arc;

use init_link_session::{handle_init_link_session, InitLinkRequest};
use serde::{Deserialize, Serialize};

use crate::common::{connection::ConnectionStream, request::DataRequest};

use super::services::Services;

mod init_link_session;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum IncomingRequestMessage {
    InitLinkSession(InitLinkRequest),
}

macro_rules! pass_request {
    ($handler:ident, $services: expr, $stream: expr, $message: expr) => {{
        let mut data_request = DataRequest::new($message, $stream);
        $handler($services, &mut data_request).await;
        data_request.response_stream
    }};
}

pub async fn handle(
    services: Arc<Services>,
    stream: ConnectionStream,
    message: IncomingRequestMessage,
) -> ConnectionStream {
    match message {
        IncomingRequestMessage::InitLinkSession(request) => {
            pass_request!(handle_init_link_session, services, stream, request)
        }
    }
}
