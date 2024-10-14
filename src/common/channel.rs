use log::error;
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

        self.tx.send(request).await.unwrap(); // TODO: FIX

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

    pub async fn send(&self, data: T) -> Result<()> {
        let request = Request {
            data,
            response_tx: None,
        };

        self.tx.send(request).await.map_err(|_| {
            error!("Failed to send request!");
            tokio::io::Error::new(tokio::io::ErrorKind::Other, "Failed to send request!")
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
    pub async fn recv(&mut self) -> Option<Request<T>> {
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
macro_rules! connect_request_with_enum {
    ($request_struct: ident, $enum: ident) => {
        impl Into<$enum> for $request_struct {
            fn into(self) -> $enum {
                $enum::$request_struct(self)
            }
        }
    };
}

#[macro_export]
macro_rules! connect_response_with_enum {
    ($response_struct: ident, $response_enum: ident) => {
        impl Into<$response_enum> for $response_struct {
            fn into(self) -> $response_enum {
                $response_enum::$response_struct(self)
            }
        }

        #[allow(unreachable_patterns)]
        impl TryFrom<$response_enum> for $response_struct {
            type Error = ();

            fn try_from(response: $response_enum) -> Result<Self, Self::Error> {
                match response {
                    $response_enum::$response_struct(response) => Ok(response),
                    _ => Err(()),
                }
            }
        }
    };
}

#[macro_export]
macro_rules! connect_request_with_response_struct {
    ($request_struct: ident, $response_struct: ident) => {
        impl crate::common::channel::DataResponse for $request_struct {
            type Response = $response_struct;
        }
    };
}

#[macro_export]
macro_rules! connect_request_with_reponse_enum {
    ($request_enum: ident, $response_enum: ident) => {
        impl RequestEnum for $request_enum {
            type ResponseEnum = $response_enum;
        }
    };
}
