use std::sync::Arc;

use init_link_session::{process_init_link, InitLinkRequest};
use serde::{Deserialize, Serialize};

use crate::common::{connection::ConnectionStream, request::DataRequest};

use super::services::Services;

mod init_link_session;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum TunnelRequestMessage {
    InitLinkSession(InitLinkRequest),
}

macro_rules! single_stream_process {
    ($handler:ident, $services: expr, $stream: expr, $message: expr) => {{
        let mut data_request = DataRequest::new($message, $stream);
        $handler($services, &mut data_request).await;
        data_request.response_stream
    }};
}

pub async fn handle(
    services: Arc<Services>,
    stream: ConnectionStream,
    message: TunnelRequestMessage,
) -> ConnectionStream {
    match message {
        TunnelRequestMessage::InitLinkSession(request) => {
            single_stream_process!(process_init_link, services, stream, request)
        }
    }
}
