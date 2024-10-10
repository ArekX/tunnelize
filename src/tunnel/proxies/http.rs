use std::ops::ControlFlow;
use std::sync::Arc;

use log::debug;
use serde::{Deserialize, Serialize};
use tokio::io::Result;
use tokio::net::TcpStream;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::common::connection::ConnectionStream;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HttpProxy {
    pub desired_name: Option<String>,
    pub forward_address: String,
}

pub struct HttpSession {
    pub session_key: Uuid,
    pub server_ddress: String,
    pub proxy: Arc<HttpProxy>,
    pub cancel_token: CancellationToken,
}

pub async fn start(session: HttpSession) -> Result<()> {
    let stream = match TcpStream::connect(session.server_ddress.clone()).await {
        Ok(stream) => stream,
        Err(e) => {
            debug!("Error connecting to server: {:?}", e);
            return Err(e);
        }
    };

    let mut connection_stream = ConnectionStream::from(stream);

    loop {
        tokio::select! {
            _ = session.cancel_token.cancelled() => {
                debug!("Http proxy session stopped.");
                return Ok(());
            }
            flow = connection_stream.wait_for_messages() => {
                match flow {
                    Ok(ControlFlow::Break(_)) => {
                        println!("Server closed the connection.");
                        return Ok(());
                    }
                    Ok(ControlFlow::Continue(_)) => {}
                    Err(e) => {
                        debug!("Error waiting for messages: {:?}", e);
                        return Err(e);
                    }
                }
            }
        }
    }

    Ok(())
}
