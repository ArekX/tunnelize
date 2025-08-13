use log::error;
use serde::Serialize;
use tokio::io::Result;
use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    oneshot,
};

pub trait RequestEnum {
    type ResponseEnum;
}

pub trait DataResponse {
    type Response;
}

pub trait Responder<T: RequestEnum> {
    fn respond<Response>(self, response: Response)
    where
        Response: Into<T::ResponseEnum>;
}

impl<T: RequestEnum> Responder<T> for Option<oneshot::Sender<T::ResponseEnum>> {
    fn respond<Response>(self, response: Response)
    where
        Response: Into<T::ResponseEnum>,
    {
        if let Some(tx) = self {
            if tx.send(response.into()).is_err() {
                error!("Failed to send response!");
            }
        } else {
            error!("No response channel to send response!");
        }
    }
}

#[derive(Debug)]
pub struct Request<T: RequestEnum> {
    pub data: T,
    response_tx: Option<oneshot::Sender<T::ResponseEnum>>,
}

impl<T: RequestEnum> Request<T> {
    pub fn assign_tx(&mut self, tx: oneshot::Sender<T::ResponseEnum>) {
        self.response_tx = Some(tx);
    }

    pub fn take_responder(&mut self) -> impl Responder<T> + use<T> {
        self.response_tx.take()
    }

    pub fn respond<Response>(&mut self, response: Response)
    where
        Response: Into<T::ResponseEnum>,
    {
        self.take_responder().respond(response);
    }
}

#[derive(Debug)]
pub struct RequestSender<T: RequestEnum> {
    tx: Sender<Request<T>>,
}

unsafe impl<T: RequestEnum> Sync for RequestSender<T> {}
unsafe impl<T: RequestEnum> Send for RequestSender<T> {}

impl<T: RequestEnum> RequestSender<T> {
    pub async fn request<Data>(&self, data: Data) -> Result<Data::Response>
    where
        Data: Into<T> + DataResponse,
        Data::Response: TryFrom<T::ResponseEnum>,
    {
        let (response_tx, response_rx) = oneshot::channel::<T::ResponseEnum>();

        let mut request = Request {
            data: data.into(),
            response_tx: None,
        };

        request.assign_tx(response_tx);

        if self.tx.send(request).await.is_err() {
            return Err(tokio::io::Error::other("Failed to send request!"));
        }

        let Ok(result) = response_rx.await else {
            return Err(tokio::io::Error::other("Failed to receive response!"));
        };

        Data::Response::try_from(result).map_err(|_| {
            tokio::io::Error::new(
                tokio::io::ErrorKind::InvalidData,
                "Failed to convert response!",
            )
        })
    }
}

impl<T: RequestEnum> Clone for RequestSender<T> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
        }
    }
}

pub struct RequestReceiver<T: RequestEnum> {
    rx: Receiver<Request<T>>,
}

impl<T: RequestEnum> RequestReceiver<T> {
    pub async fn wait_for_requests(&mut self) -> Option<Request<T>> {
        self.rx.recv().await
    }

    pub fn close(&mut self) {
        self.rx.close();
    }
}

pub fn create_channel<T: RequestEnum>() -> (RequestSender<T>, RequestReceiver<T>) {
    let (tx, rx) = mpsc::channel::<Request<T>>(100);

    let sender = RequestSender { tx };
    let receiver = RequestReceiver { rx };

    (sender, receiver)
}

#[macro_export]
macro_rules! create_channel_enum {
    ($request_enum: ident -> $response_enum: ident, {
        $($request_type: ident -> $response_type: ident),*
    }) => {
        #[allow(clippy::enum_variant_names)]
        #[derive(Debug)]
        pub enum $request_enum {
        $(
            $request_type($request_type)
        ),*
        }

        impl $crate::common::channel::RequestEnum for $request_enum {
            type ResponseEnum = $response_enum;
        }

        #[allow(clippy::enum_variant_names)]
        #[derive(Debug)]
        pub enum $response_enum {
            InvalidResponse,
        $(
            $response_type($response_type)
        ),*

        }

        impl From<$crate::common::channel::InvalidResponse> for $response_enum {
            fn from(_val: $crate::common::channel::InvalidResponse) -> Self {
                $response_enum::InvalidResponse
            }
        }

        $(

            impl Into<$request_enum> for $request_type {
                fn into(self) -> $request_enum {
                    $request_enum::$request_type(self)
                }
            }

            impl $crate::common::channel::DataResponse for $request_type {
                type Response = $response_type;
            }

            impl Into<$response_enum> for $response_type {
                fn into(self) -> $response_enum {
                    $response_enum::$response_type(self)
                }
            }

            #[allow(unreachable_patterns)]
            impl TryFrom<$response_enum> for $response_type {
                type Error = ();

                fn try_from(response: $response_enum) -> Result<Self, Self::Error> {
                    match response {
                        $response_enum::$response_type(response) => Ok(response),
                        _ => Err(()),
                    }
                }
            }


        )*
    };
}

#[derive(Debug, Clone, Serialize)]
pub struct OkResponse;

#[derive(Debug, Clone)]
pub struct InvalidResponse;
