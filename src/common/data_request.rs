use super::connection::ConnectionStream;
use serde::Serialize;

pub trait DataRequestResponse: Serialize {
    type ResponseMessage: Serialize;
}

#[derive(Debug)]
pub struct DataRequest<RequestMessage: DataRequestResponse, Stream = ConnectionStream> {
    pub data: RequestMessage,
    pub response_stream: Stream,
}

impl<RequestMessage: DataRequestResponse, Stream> DataRequest<RequestMessage, Stream> {
    pub fn new(data: RequestMessage, response_stream: Stream) -> Self {
        Self {
            data,
            response_stream,
        }
    }
}

#[macro_export]
macro_rules! connect_data_response {
    ($request: ident, $response: ident) => {
        impl crate::common::data_request::DataRequestResponse for $request {
            type ResponseMessage = $response;
        }
    };
}
