use std::{
    error::Error,
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};

use http::Uri;
use hyper_util::{
    client::legacy::connect::{Connected, Connection},
    rt::TokioIo,
};
use log::{debug, info};
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    sync::{OwnedSemaphorePermit, RwLock, Semaphore, oneshot},
    time::timeout,
};
use tower_service::Service;
use uuid::Uuid;

use crate::{
    common::connection::TransportStream,
    server::{
        services::{Client, Services},
        session::messages::{ClientLinkRequest, ClientLinkResponse},
    },
};

use super::tunnel_host::TunnelHost;

/// How long the link session may take to hand the established stream over
/// once the tunnel has accepted the link.
const HANDOFF_TIMEOUT: Duration = Duration::from_secs(10);

type BoxError = Box<dyn Error + Send + Sync>;

/// Opens tunnel links for the HTTP client pool.
///
/// The pool asks for a connection per request URI authority, which the proxy
/// sets to the tunnel hostname, so every pooled connection is a link to the
/// tunnel that hostname belongs to. Idle links are kept and reused across
/// requests, keeping the local origin's keep-alive connection alive as well.
#[derive(Clone)]
pub struct LinkConnector {
    inner: Arc<Inner>,
}

struct Inner {
    services: Arc<Services>,
    endpoint_name: String,
    tunnel_host: Arc<RwLock<TunnelHost>>,
    /// Caps the number of simultaneously open links. Requests beyond the cap
    /// wait for a link to free up instead of failing.
    link_limit: Arc<Semaphore>,
}

impl LinkConnector {
    pub fn new(
        services: Arc<Services>,
        endpoint_name: String,
        tunnel_host: Arc<RwLock<TunnelHost>>,
        max_links: usize,
    ) -> Self {
        Self {
            inner: Arc::new(Inner {
                services,
                endpoint_name,
                tunnel_host,
                link_limit: Arc::new(Semaphore::new(max_links)),
            }),
        }
    }
}

impl Service<Uri> for LinkConnector {
    type Response = TokioIo<LinkIo>;
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, uri: Uri) -> Self::Future {
        let inner = self.inner.clone();

        Box::pin(async move { inner.open_link(uri).await.map(TokioIo::new) })
    }
}

impl Inner {
    async fn open_link(&self, uri: Uri) -> Result<LinkIo, BoxError> {
        let hostname = uri.host().ok_or("Request is missing a hostname")?;

        let session = self
            .tunnel_host
            .read()
            .await
            .get_session(hostname)
            .ok_or("No tunnel is assigned for the requested hostname")?;

        let permit = self.link_limit.clone().acquire_owned().await?;

        let client_id = Uuid::new_v4();
        let (client, handoff_rx) = Client::new_handoff(client_id, self.endpoint_name.clone());

        self.services
            .get_client_manager()
            .await
            .subscribe_client(client)
            .map_err(|(e, _)| e)?;

        let link_request = ClientLinkRequest {
            client_id,
            proxy_id: session.proxy_id,
        };

        match self
            .services
            .send_tunnel_request(&session.tunnel_id, link_request)
            .await
        {
            Ok(ClientLinkResponse::Accepted) => {}
            Ok(ClientLinkResponse::Rejected { reason }) => {
                debug!(
                    "Client ID '{}' rejected by tunnel ID '{}': {}",
                    client_id, session.tunnel_id, reason
                );
                self.cancel_client(&client_id).await;
                return Err(reason.into());
            }
            Err(e) => {
                self.cancel_client(&client_id).await;
                return Err(format!("Failed to link client to tunnel: {e}").into());
            }
        }

        let handoff = match timeout(HANDOFF_TIMEOUT, handoff_rx).await {
            Ok(Ok(handoff)) => handoff,
            _ => {
                self.cancel_client(&client_id).await;
                return Err("Tunnel link was accepted but never handed over".into());
            }
        };

        let stream = handoff.stream.into_transport_stream()?;

        info!(
            "Client ID '{}' linked to tunnel ID '{}'",
            client_id, session.tunnel_id
        );

        Ok(LinkIo {
            stream,
            _release: handoff.release,
            _permit: permit,
        })
    }

    async fn cancel_client(&self, client_id: &Uuid) {
        self.services
            .get_client_manager()
            .await
            .cancel_client(client_id, &None)
            .await;
    }
}

/// A pooled link stream. Dropping it ends the link session on the server and
/// frees its slot in the link limit.
pub struct LinkIo {
    stream: TransportStream,
    _release: oneshot::Sender<()>,
    _permit: OwnedSemaphorePermit,
}

impl Connection for LinkIo {
    fn connected(&self) -> Connected {
        Connected::new()
    }
}

impl AsyncRead for LinkIo {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.get_mut().stream).poll_read(cx, buf)
    }
}

impl AsyncWrite for LinkIo {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.get_mut().stream).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.get_mut().stream).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.get_mut().stream).poll_shutdown(cx)
    }
}
