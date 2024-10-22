use tokio::io::Result;

use crate::{
    common::channel::{InvalidResponse, OkResponse, Request},
    server::endpoints::messages::{EndpointChannelRequest, RemoveTunnelRequest},
};

pub async fn handle(mut request: Request<EndpointChannelRequest>) -> Result<()> {
    match &request.data {
        EndpointChannelRequest::RemoveTunnelRequest(RemoveTunnelRequest { .. }) => {
            request.respond(OkResponse).await;
        }
        _ => {
            request.respond(InvalidResponse).await;
        }
    }

    Ok(())
}
