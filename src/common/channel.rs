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

#[derive(Debug)]
pub struct Request<T: RequestEnum> {
    pub data: T,
    response_tx: Option<oneshot::Sender<T::ResponseEnum>>,
}

impl<T: RequestEnum> Request<T> {
    pub fn assign_tx(&mut self, tx: oneshot::Sender<T::ResponseEnum>) {
        self.response_tx = Some(tx);
    }

    pub async fn respond<Response>(&mut self, response: Response)
    where
        Response: Into<T::ResponseEnum>,
    {
        if let Some(tx) = self.response_tx.take() {
            if let Err(_) = tx.send(response.into()) {
                error!("Failed to send response!");
            }
        } else {
            error!("No response channel to send response!");
        }
    }
}

#[derive(Debug)]
pub struct RequestSender<T: RequestEnum> {
    tx: Sender<Request<T>>,
}

unsafe impl<T: RequestEnum> Sync for RequestSender<T> {}
unsafe impl<T: RequestEnum> Send for RequestSender<T> {}

impl<T: RequestEnum> RequestSender<T> {
    pub async fn request<Data: DataResponse>(&self, data: Data) -> Result<Data::Response>
    where
        Data: Into<T>,
        Data::Response: TryFrom<T::ResponseEnum>,
    {
        let (response_tx, response_rx) = oneshot::channel::<T::ResponseEnum>();

        let mut request = Request {
            data: data.into(),
            response_tx: None,
        };

        request.assign_tx(response_tx);

        if let Err(_) = self.tx.send(request).await {
            return Err(tokio::io::Error::new(
                tokio::io::ErrorKind::Other,
                "Failed to send request!",
            ));
        }

        let Ok(result) = response_rx.await else {
            return Err(tokio::io::Error::new(
                tokio::io::ErrorKind::Other,
                "Failed to receive response!",
            ));
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
        #[derive(Debug)]
        pub enum $request_enum {
        $(
            $request_type($request_type)
        ),*
        }

        impl $crate::common::channel::RequestEnum for $request_enum {
            type ResponseEnum = $response_enum;
        }

        #[derive(Debug)]
        pub enum $response_enum {
            InvalidResponse,
        $(
            $response_type($response_type)
        ),*

        }

        impl Into<$response_enum> for $crate::common::channel::InvalidResponse {
            fn into(self) -> $response_enum {
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
