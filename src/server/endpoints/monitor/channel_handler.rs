use tokio::io::Result;

use crate::{
    common::channel::{InvalidResponse, OkResponse, Request},
    server::endpoints::messages::EndpointChannelRequest,
};

pub async fn handle(mut request: Request<EndpointChannelRequest>) -> Result<()> {
    match &request.data {
        EndpointChannelRequest::RemoveTunnelRequest(_) => {
            request.respond(OkResponse);
        }
        _ => {
            request.respond(InvalidResponse);
        }
    }

    Ok(())
}
