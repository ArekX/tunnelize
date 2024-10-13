use std::future::Future;

use log::{debug, error};
use tokio::sync::{mpsc, oneshot};

pub trait ChannelRequestResponse {
    type ResponseMessage;
}

#[derive(Debug)]
pub struct ChannelRequest<RequestMessage: ChannelRequestResponse> {
    pub data: RequestMessage,
    response_tx: Option<oneshot::Sender<RequestMessage::ResponseMessage>>,
}

impl<RequestMessage: ChannelRequestResponse> ChannelRequest<RequestMessage> {
    pub fn new(data: RequestMessage) -> Self {
        Self {
            data,
            response_tx: None,
        }
    }

    pub fn assign_response_tx(
        &mut self,
        response_tx: oneshot::Sender<RequestMessage::ResponseMessage>,
    ) {
        self.response_tx = Some(response_tx);
    }

    pub fn respond(&mut self, response: RequestMessage::ResponseMessage) {
        if let Some(tx) = self.response_tx.take() {
            if let Err(_) = tx.send(response) {
                error!("Failed to send response!");
            }
        } else {
            debug!("No response channel to send response!");
        }
    }
}

#[macro_export]
macro_rules! connect_channel_response {
    ($request: ident, $response: ident) => {
        impl crate::common::channel_request::ChannelRequestResponse for $request {
            type ResponseMessage = $response;
        }
    };
}

#[macro_export]
macro_rules! map_request_enum {
    ($enum: ident, { $($struct:ident => $enum_name:ident),+ }) => {
        $(
            impl From<ChannelRequest<$struct>> for $enum {
                fn from(request: ChannelRequest<$struct>) -> Self {
                    Self::$enum_name(request)
                }
            }
        )+
    };
}

pub async fn send_channel_request<SenderType, T>(
    sender_tx: mpsc::Sender<SenderType>,
    mut request: ChannelRequest<T>,
) -> tokio::io::Result<T::ResponseMessage>
where
    SenderType: From<ChannelRequest<T>>,
    T: ChannelRequestResponse,
{
    let (result_tx, result_rx) = oneshot::channel::<T::ResponseMessage>();

    request.assign_response_tx(result_tx);

    match sender_tx.send(SenderType::from(request)).await {
        Ok(_) => {}
        Err(_) => {
            error!("Failed to send request!");
            return Err(tokio::io::Error::new(
                tokio::io::ErrorKind::Other,
                "Failed to send request",
            ));
        }
    }

    match result_rx.await {
        Ok(response) => Ok(response),
        Err(e) => {
            error!("Failed to receive response: {}", e);
            Err(tokio::io::Error::new(
                tokio::io::ErrorKind::Other,
                "Failed to receive response",
            ))
        }
    }
}
