use std::{convert::Infallible, error::Error, io, net::SocketAddr, sync::Arc, time::Duration};

use base64::{Engine as _, engine::general_purpose};
use bytes::Bytes;
use http::{
    HeaderMap, HeaderName, HeaderValue, Request, Response, StatusCode, Uri, Version,
    header::{self, CONNECTION, HOST, UPGRADE},
};
use http_body_util::{BodyExt, Full, combinators::BoxBody};
use hyper::{body::Incoming, service::service_fn};
use hyper_util::{
    client::legacy::Client,
    rt::{TokioExecutor, TokioIo, TokioTimer},
    server::conn::auto,
};
use log::{debug, error};
use tokio::sync::RwLock;

use crate::{
    common::{connection::Connection, text::is_constant_time_equals},
    server::services::Services,
};

use super::{
    configuration::{AuthorizeUser, HttpEndpointConfig},
    link_connector::LinkConnector,
    tunnel_host::TunnelHost,
};

/// How long an idle link stays pooled before it is closed.
const POOL_IDLE_TIMEOUT: Duration = Duration::from_secs(90);

type ProxyBody = BoxBody<Bytes, hyper::Error>;

/// Serves browser connections (HTTP/1.1 or HTTP/2) and forwards each request
/// over a pooled tunnel link as HTTP/1.1.
pub struct HttpProxy {
    config: Arc<HttpEndpointConfig>,
    tunnel_host: Arc<RwLock<TunnelHost>>,
    client: Client<LinkConnector, Incoming>,
}

impl HttpProxy {
    pub fn new(
        services: Arc<Services>,
        endpoint_name: String,
        config: Arc<HttpEndpointConfig>,
        tunnel_host: Arc<RwLock<TunnelHost>>,
    ) -> Self {
        let connector = LinkConnector::new(
            services.clone(),
            endpoint_name,
            tunnel_host.clone(),
            services.get_config().get_max_clients(),
        );

        let client = Client::builder(TokioExecutor::new())
            .pool_idle_timeout(POOL_IDLE_TIMEOUT)
            .pool_max_idle_per_host(usize::MAX)
            .pool_timer(TokioTimer::new())
            .build(connector);

        Self {
            config,
            tunnel_host,
            client,
        }
    }

    pub async fn serve(
        self: &Arc<Self>,
        connection: Connection,
        peer: SocketAddr,
    ) -> io::Result<()> {
        let stream = connection.into_transport_stream()?;
        let proxy = self.clone();

        let service = service_fn(move |request| {
            let proxy = proxy.clone();
            async move { Ok::<_, Infallible>(proxy.handle(request, peer).await) }
        });

        let mut builder = auto::Builder::new(TokioExecutor::new());
        builder
            .http1()
            .timer(TokioTimer::new())
            .header_read_timeout(self.header_read_timeout());
        builder.http2().timer(TokioTimer::new());

        builder
            .serve_connection_with_upgrades(TokioIo::new(stream), service)
            .await
            .map_err(io::Error::other)
    }

    /// Answers a plain HTTP request that reached a TLS endpoint with a redirect
    /// to the HTTPS address.
    pub async fn serve_tls_redirect(&self, connection: Connection) -> io::Result<()> {
        let stream = connection.into_transport_stream()?;
        let port = self.config.port;

        let service = service_fn(move |request: Request<Incoming>| async move {
            let response = match request_authority(&request) {
                Some(authority) => {
                    let path = request
                        .uri()
                        .path_and_query()
                        .map(|path| path.as_str())
                        .unwrap_or("/");
                    let location = format!("https://{}:{}{}", strip_port(&authority), port, path);

                    Response::builder()
                        .status(StatusCode::MOVED_PERMANENTLY)
                        .header(header::LOCATION, location)
                        .header(CONNECTION, "close")
                        .body(full_body(""))
                        .expect("valid redirect response")
                }
                None => text_response(StatusCode::BAD_REQUEST, "Host header is missing"),
            };

            Ok::<_, Infallible>(response)
        });

        hyper::server::conn::http1::Builder::new()
            .timer(TokioTimer::new())
            .header_read_timeout(self.header_read_timeout())
            .serve_connection(TokioIo::new(stream), service)
            .await
            .map_err(io::Error::other)
    }

    fn header_read_timeout(&self) -> Duration {
        Duration::from_secs(self.config.get_max_client_input_wait_secs())
    }

    async fn handle(
        &self,
        mut request: Request<Incoming>,
        peer: SocketAddr,
    ) -> Response<ProxyBody> {
        let Some(authority) = request_authority(&request) else {
            return text_response(StatusCode::BAD_REQUEST, "Host header is missing");
        };

        if let Some(user) = &self.config.require_authorization
            && !is_authorized(request.headers(), user)
        {
            return unauthorized_response(user);
        }

        let hostname = strip_port(&authority).to_owned();

        if self
            .tunnel_host
            .read()
            .await
            .get_session(&hostname)
            .is_none()
        {
            return text_response(
                StatusCode::BAD_GATEWAY,
                "No tunnel is assigned for the requested hostname",
            );
        }

        let server_upgrade =
            has_upgrade(request.headers()).then(|| hyper::upgrade::on(&mut request));

        let upstream =
            match build_upstream_request(request, &hostname, &authority, peer, self.scheme()) {
                Ok(upstream) => upstream,
                Err(e) => {
                    debug!("Rejecting malformed request for '{}': {}", authority, e);
                    return text_response(StatusCode::BAD_REQUEST, "Malformed request");
                }
            };

        let mut response = match self.client.request(upstream).await {
            Ok(response) => response,
            Err(e) => {
                let reason = error_chain(&e);
                error!("Failed to proxy request for '{}': {}", hostname, reason);
                return text_response(StatusCode::BAD_GATEWAY, &reason);
            }
        };

        if response.status() == StatusCode::SWITCHING_PROTOCOLS
            && let Some(server_upgrade) = server_upgrade
        {
            let client_upgrade = hyper::upgrade::on(&mut response);

            tokio::spawn(async move {
                match tokio::try_join!(server_upgrade, client_upgrade) {
                    Ok((client_side, tunnel_side)) => {
                        let mut client_side = TokioIo::new(client_side);
                        let mut tunnel_side = TokioIo::new(tunnel_side);

                        if let Err(e) =
                            tokio::io::copy_bidirectional(&mut client_side, &mut tunnel_side).await
                        {
                            debug!("Upgraded connection ended: {}", e);
                        }
                    }
                    Err(e) => debug!("Failed to upgrade connection: {}", e),
                }
            });
        }

        let switching_protocols = response.status() == StatusCode::SWITCHING_PROTOCOLS;
        strip_hop_by_hop_headers(response.headers_mut(), switching_protocols);

        response.map(|body| body.boxed())
    }

    fn scheme(&self) -> &'static str {
        match self.config.encryption {
            Some(_) => "https",
            None => "http",
        }
    }
}

/// Builds the HTTP/1.1 request sent over the tunnel link. The URI authority is
/// set to the tunnel hostname, which is what the connection pool keys on.
fn build_upstream_request<B>(
    request: Request<B>,
    hostname: &str,
    authority: &str,
    peer: SocketAddr,
    scheme: &str,
) -> Result<Request<B>, Box<dyn Error + Send + Sync>> {
    let (mut parts, body) = request.into_parts();

    let path = parts
        .uri
        .path_and_query()
        .map(|path| path.as_str().to_owned())
        .unwrap_or_else(|| "/".to_owned());

    parts.uri = Uri::builder()
        .scheme("http")
        .authority(hostname)
        .path_and_query(path)
        .build()?;
    parts.version = Version::HTTP_11;

    let upgrade = strip_hop_by_hop_headers(&mut parts.headers, true);

    if upgrade {
        parts
            .headers
            .insert(CONNECTION, HeaderValue::from_static("Upgrade"));
    }

    parts
        .headers
        .insert(HOST, HeaderValue::from_str(authority)?);

    let forwarded_for = HeaderName::from_static("x-forwarded-for");
    let forwarded_for_value = match parts.headers.get(&forwarded_for) {
        Some(existing) => format!("{}, {}", existing.to_str().unwrap_or(""), peer.ip()),
        None => peer.ip().to_string(),
    };
    parts
        .headers
        .insert(forwarded_for, HeaderValue::from_str(&forwarded_for_value)?);
    parts.headers.insert(
        HeaderName::from_static("x-forwarded-proto"),
        HeaderValue::from_static(if scheme == "https" { "https" } else { "http" }),
    );

    Ok(Request::from_parts(parts, body))
}

/// The host the client asked for: the `:authority` of an HTTP/2 request or the
/// `Host` header of an HTTP/1.1 one, including the port if one was given.
fn request_authority<B>(request: &Request<B>) -> Option<String> {
    if let Some(authority) = request.uri().authority() {
        return Some(authority.to_string());
    }

    request
        .headers()
        .get(HOST)
        .and_then(|host| host.to_str().ok())
        .map(|host| host.trim().to_owned())
        .filter(|host| !host.is_empty())
}

fn strip_port(authority: &str) -> &str {
    if let Some(end) = authority.strip_prefix('[').and_then(|rest| rest.find(']')) {
        return &authority[1..=end];
    }

    authority
        .split_once(':')
        .map(|(host, _)| host)
        .unwrap_or(authority)
}

fn has_upgrade(headers: &HeaderMap) -> bool {
    headers.contains_key(UPGRADE)
}

/// Removes the headers that only apply to one hop, plus any headers named in
/// `Connection`. Returns whether an `Upgrade` header was present; it is kept
/// when `keep_upgrade` is set so that WebSocket handshakes go through.
fn strip_hop_by_hop_headers(headers: &mut HeaderMap, keep_upgrade: bool) -> bool {
    let had_upgrade = headers.contains_key(UPGRADE);

    let named_in_connection: Vec<HeaderName> = headers
        .get_all(CONNECTION)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(','))
        .filter_map(|name| HeaderName::from_bytes(name.trim().as_bytes()).ok())
        .collect();

    for name in named_in_connection {
        if !(keep_upgrade && name == UPGRADE) {
            headers.remove(name);
        }
    }

    const HOP_BY_HOP: [HeaderName; 7] = [
        CONNECTION,
        header::PROXY_AUTHENTICATE,
        header::PROXY_AUTHORIZATION,
        header::TE,
        header::TRAILER,
        header::TRANSFER_ENCODING,
        HeaderName::from_static("keep-alive"),
    ];

    for name in HOP_BY_HOP {
        headers.remove(name);
    }

    if !keep_upgrade {
        headers.remove(UPGRADE);
    }

    had_upgrade
}

fn is_authorized(headers: &HeaderMap, user: &AuthorizeUser) -> bool {
    let expected = general_purpose::STANDARD.encode(format!("{}:{}", user.username, user.password));

    headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split_whitespace().last())
        .map(|credentials| is_constant_time_equals(credentials, &expected))
        .unwrap_or(false)
}

fn unauthorized_response(user: &AuthorizeUser) -> Response<ProxyBody> {
    let realm = user
        .realm
        .as_deref()
        .unwrap_or("Production")
        .replace('"', "");

    let mut response = text_response(
        StatusCode::UNAUTHORIZED,
        "Access to the requested endpoint is not authorized. Please provide valid credentials.",
    );

    if let Ok(value) = HeaderValue::from_str(&format!("Basic realm=\"{realm}\"")) {
        response
            .headers_mut()
            .insert(header::WWW_AUTHENTICATE, value);
    }

    response
}

fn full_body(text: &str) -> ProxyBody {
    Full::new(Bytes::copy_from_slice(text.as_bytes()))
        .map_err(|never| match never {})
        .boxed()
}

fn text_response(status: StatusCode, text: &str) -> Response<ProxyBody> {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "text/plain")
        .body(full_body(text))
        .expect("valid text response")
}

fn error_chain(error: &dyn Error) -> String {
    let mut message = error.to_string();
    let mut source = error.source();

    while let Some(cause) = source {
        message.push_str(": ");
        message.push_str(&cause.to_string());
        source = cause.source();
    }

    message
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user(realm: Option<&str>) -> AuthorizeUser {
        AuthorizeUser {
            realm: realm.map(str::to_owned),
            username: "user".to_owned(),
            password: "pass".to_owned(),
        }
    }

    #[test]
    fn test_request_authority_prefers_uri_authority() {
        let request = Request::builder()
            .uri("https://tunnel.example.com:3457/path")
            .header(HOST, "ignored")
            .body(())
            .unwrap();
        assert_eq!(
            request_authority(&request).as_deref(),
            Some("tunnel.example.com:3457")
        );

        let request = Request::builder()
            .uri("/path")
            .header(HOST, " tunnel.example.com ")
            .body(())
            .unwrap();
        assert_eq!(
            request_authority(&request).as_deref(),
            Some("tunnel.example.com")
        );

        let request = Request::builder().uri("/path").body(()).unwrap();
        assert_eq!(request_authority(&request), None);
    }

    #[test]
    fn test_strip_port() {
        assert_eq!(strip_port("example.com:3457"), "example.com");
        assert_eq!(strip_port("example.com"), "example.com");
        assert_eq!(strip_port("[::1]:3457"), "::1");
        assert_eq!(strip_port("[::1]"), "::1");
    }

    #[test]
    fn test_strip_hop_by_hop_headers() {
        let mut headers = HeaderMap::new();
        headers.insert(CONNECTION, HeaderValue::from_static("keep-alive, x-custom"));
        headers.insert("x-custom", HeaderValue::from_static("1"));
        headers.insert("keep-alive", HeaderValue::from_static("timeout=5"));
        headers.insert(
            header::TRANSFER_ENCODING,
            HeaderValue::from_static("chunked"),
        );
        headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("text/plain"));

        assert!(!strip_hop_by_hop_headers(&mut headers, true));

        assert!(headers.get(CONNECTION).is_none());
        assert!(headers.get("x-custom").is_none());
        assert!(headers.get("keep-alive").is_none());
        assert!(headers.get(header::TRANSFER_ENCODING).is_none());
        assert!(headers.get(header::CONTENT_TYPE).is_some());
    }

    #[test]
    fn test_strip_hop_by_hop_headers_keeps_upgrade_when_asked() {
        let mut headers = HeaderMap::new();
        headers.insert(CONNECTION, HeaderValue::from_static("Upgrade"));
        headers.insert(UPGRADE, HeaderValue::from_static("websocket"));

        assert!(strip_hop_by_hop_headers(&mut headers.clone(), true));
        let mut kept = headers.clone();
        strip_hop_by_hop_headers(&mut kept, true);
        assert_eq!(kept.get(UPGRADE).unwrap(), "websocket");

        strip_hop_by_hop_headers(&mut headers, false);
        assert!(headers.get(UPGRADE).is_none());
    }

    #[test]
    fn test_build_upstream_request() {
        let request = Request::builder()
            .method("POST")
            .version(Version::HTTP_2)
            .uri("https://tunnel.example.com:3457/api?x=1")
            .header("x-forwarded-for", "10.0.0.1")
            .header(CONNECTION, "Upgrade")
            .header(UPGRADE, "websocket")
            .body(())
            .unwrap();
        let upstream = build_upstream_request(
            request,
            "tunnel.example.com",
            "tunnel.example.com:3457",
            "192.168.1.2:5000".parse().unwrap(),
            "https",
        )
        .unwrap();

        assert_eq!(
            upstream.uri().to_string(),
            "http://tunnel.example.com/api?x=1"
        );
        assert_eq!(upstream.version(), Version::HTTP_11);
        assert_eq!(
            upstream.headers().get(HOST).unwrap(),
            "tunnel.example.com:3457"
        );
        assert_eq!(upstream.headers().get(CONNECTION).unwrap(), "Upgrade");
        assert_eq!(upstream.headers().get(UPGRADE).unwrap(), "websocket");
        assert_eq!(
            upstream.headers().get("x-forwarded-for").unwrap(),
            "10.0.0.1, 192.168.1.2"
        );
        assert_eq!(
            upstream.headers().get("x-forwarded-proto").unwrap(),
            "https"
        );
    }

    #[test]
    fn test_is_authorized() {
        let mut headers = HeaderMap::new();
        assert!(!is_authorized(&headers, &user(None)));

        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Basic dXNlcjpwYXNz"),
        );
        assert!(is_authorized(&headers, &user(None)));

        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Basic bm9wZTpub3Bl"),
        );
        assert!(!is_authorized(&headers, &user(None)));
    }

    #[test]
    fn test_unauthorized_response_sets_realm() {
        let response = unauthorized_response(&user(Some("My \"Realm\"")));
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(
            response.headers().get(header::WWW_AUTHENTICATE).unwrap(),
            "Basic realm=\"My Realm\""
        );

        let response = unauthorized_response(&user(None));
        assert_eq!(
            response.headers().get(header::WWW_AUTHENTICATE).unwrap(),
            "Basic realm=\"Production\""
        );
    }
}
