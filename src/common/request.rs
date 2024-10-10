use super::connection::ConnectionStream;
use serde::Serialize;

pub trait DataRequestResponse: Serialize {
    type ResponseMessage: Serialize;
}

pub struct DataRequest<RequestMessage: DataRequestResponse> {
    pub data: RequestMessage,
    pub response_stream: ConnectionStream,
}

impl<RequestMessage: DataRequestResponse> DataRequest<RequestMessage> {
    pub fn new(data: RequestMessage, response_stream: ConnectionStream) -> Self {
        Self {
            data,
            response_stream,
        }
    }

    pub async fn respond(&mut self, response: &RequestMessage::ResponseMessage) {
        self.response_stream.respond_message(response).await;
    }
}

#[macro_export]
macro_rules! connect_data_response {
    ($request: ident, $response: ident) => {
        impl DataRequestResponse for $request {
            type ResponseMessage = $response;
        }
    };
}
