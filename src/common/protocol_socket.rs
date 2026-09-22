use std::{
    collections::HashSet,
    io::{Error, ErrorKind},
    net::{IpAddr, SocketAddr},
    time::Duration,
};

use log::debug;
use tokio::{io::Result, net::TcpStream, task::JoinSet, time::timeout};

/// How long a DNS lookup may take before it is treated as failed.
const RESOLVE_TIMEOUT: Duration = Duration::from_secs(10);

/// How long a single connection attempt may take. Without this a SYN that is
/// silently dropped (firewalled host, broken IPv6 route, stale AAAA record) sits
/// in the OS connect timeout for 75+ seconds before the next address is tried.
const CONNECT_ATTEMPT_TIMEOUT: Duration = Duration::from_secs(10);

/// Head start an attempt gets before the next candidate is started alongside it
/// ("Happy Eyeballs", RFC 8305). Whichever attempt connects first wins.
const CONNECT_STAGGER_DELAY: Duration = Duration::from_millis(250);

pub trait ProtocolSocket
where
    Self: Sized,
{
    type Socket: Send + 'static;
    type Context: Clone + Send + Sync + 'static;

    fn connect(
        address: SocketAddr,
        context: Self::Context,
    ) -> impl Future<Output = Result<Self::Socket>> + Send;
}

impl ProtocolSocket for TcpStream {
    type Socket = TcpStream;
    type Context = ();

    async fn connect(address: SocketAddr, _context: Self::Context) -> Result<Self::Socket> {
        TcpStream::connect(address).await
    }
}

/// Turns an IPv4-mapped IPv6 address (`::ffff:1.2.3.4`) into the plain IPv4
/// address it stands for. Such addresses always travel over IPv4, but going
/// through an IPv6 socket for them fails or hangs on hosts without IPv6.
pub fn unmap_socket_addr(address: SocketAddr) -> SocketAddr {
    match address {
        SocketAddr::V6(v6) => match v6.ip().to_ipv4_mapped() {
            Some(v4) => SocketAddr::new(IpAddr::V4(v4), v6.port()),
            None => address,
        },
        SocketAddr::V4(_) => address,
    }
}

/// Orders resolved addresses into the sequence they are tried in: mapped
/// addresses unwrapped, duplicates removed, IPv4 before IPv6.
///
/// IPv4 goes first because it is the transport most tunnel hosts can actually
/// reach; IPv6 candidates still get tried shortly after, so a host that is only
/// reachable over IPv6 keeps working.
pub fn order_candidates(addresses: impl IntoIterator<Item = SocketAddr>) -> Vec<SocketAddr> {
    let mut seen = HashSet::new();
    let mut ipv4 = Vec::new();
    let mut ipv6 = Vec::new();

    for address in addresses {
        let address = unmap_socket_addr(address);

        if !seen.insert(address) {
            continue;
        }

        match address {
            SocketAddr::V4(_) => ipv4.push(address),
            SocketAddr::V6(_) => ipv6.push(address),
        }
    }

    ipv4.extend(ipv6);
    ipv4
}

/// Resolves `address:port` without blocking the async runtime and without
/// waiting on a stuck resolver forever. `address` may be a hostname, an IPv4
/// literal or an IPv6 literal (with or without brackets).
pub async fn resolve_address(address: &str, port: u16) -> Result<Vec<SocketAddr>> {
    let host = address.trim_start_matches('[').trim_end_matches(']');

    let resolved = match timeout(RESOLVE_TIMEOUT, tokio::net::lookup_host((host, port))).await {
        Ok(Ok(resolved)) => resolved,
        Ok(Err(e)) => return Err(e),
        Err(_) => {
            return Err(Error::new(
                ErrorKind::TimedOut,
                format!("Timed out while resolving address: {address}"),
            ));
        }
    };

    let candidates = order_candidates(resolved);

    if candidates.is_empty() {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("Could not resolve address: {address}:{port}"),
        ));
    }

    Ok(candidates)
}

pub async fn connect_to_address<Protocol: ProtocolSocket>(
    address: &str,
    port: u16,
    context: Protocol::Context,
) -> Result<(Protocol::Socket, SocketAddr)> {
    let candidates = resolve_address(address, port).await?;

    connect_to_candidates::<Protocol>(candidates, context).await
}

/// Connects to the first candidate that answers. Attempts are started in order,
/// each one a short stagger after the previous, and every attempt has its own
/// timeout, so one unreachable address only ever costs the stagger delay.
pub async fn connect_to_candidates<Protocol: ProtocolSocket>(
    candidates: Vec<SocketAddr>,
    context: Protocol::Context,
) -> Result<(Protocol::Socket, SocketAddr)> {
    let mut remaining = candidates.into_iter().peekable();
    let mut attempts: JoinSet<Result<(Protocol::Socket, SocketAddr)>> = JoinSet::new();
    let mut last_error = None;

    loop {
        if attempts.is_empty() {
            match remaining.next() {
                Some(address) => spawn_attempt::<Protocol>(&mut attempts, address, context.clone()),
                None => break,
            }
        }

        tokio::select! {
            Some(joined) = attempts.join_next() => {
                match joined {
                    Ok(Ok(connected)) => {
                        attempts.abort_all();
                        return Ok(connected);
                    }
                    Ok(Err(e)) => last_error = Some(e),
                    Err(e) => last_error = Some(Error::other(e)),
                }
            }
            _ = tokio::time::sleep(CONNECT_STAGGER_DELAY), if remaining.peek().is_some() => {
                if let Some(address) = remaining.next() {
                    spawn_attempt::<Protocol>(&mut attempts, address, context.clone());
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| Error::other("No address available to connect to")))
}

fn spawn_attempt<Protocol: ProtocolSocket>(
    attempts: &mut JoinSet<Result<(Protocol::Socket, SocketAddr)>>,
    address: SocketAddr,
    context: Protocol::Context,
) {
    attempts.spawn(async move {
        match timeout(CONNECT_ATTEMPT_TIMEOUT, Protocol::connect(address, context)).await {
            Ok(Ok(socket)) => Ok((socket, address)),
            Ok(Err(e)) => {
                debug!("Failed to connect to {address}: {e}");
                Err(e)
            }
            Err(_) => {
                debug!("Timed out while connecting to {address}");
                Err(Error::new(
                    ErrorKind::TimedOut,
                    format!("Timed out while connecting to {address}"),
                ))
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use std::{net::Ipv6Addr, time::Instant};

    use tokio::net::TcpListener;

    use super::*;

    fn addr(text: &str) -> SocketAddr {
        text.parse().unwrap()
    }

    #[test]
    fn test_unmap_socket_addr_unwraps_ipv4_mapped() {
        assert_eq!(
            unmap_socket_addr(addr("[::ffff:54.152.206.47]:3456")),
            addr("54.152.206.47:3456")
        );
        assert_eq!(
            unmap_socket_addr(addr("[2001:db8::1]:1")),
            addr("[2001:db8::1]:1")
        );
        assert_eq!(unmap_socket_addr(addr("10.0.0.1:1")), addr("10.0.0.1:1"));
    }

    #[test]
    fn test_order_candidates_prefers_ipv4_and_deduplicates() {
        let ordered = order_candidates([
            addr("[2001:db8::1]:1"),
            addr("[::ffff:10.0.0.1]:1"),
            addr("10.0.0.2:1"),
            addr("10.0.0.1:1"),
            addr("[2001:db8::1]:1"),
        ]);

        assert_eq!(
            ordered,
            vec![
                addr("10.0.0.1:1"),
                addr("10.0.0.2:1"),
                addr("[2001:db8::1]:1")
            ]
        );
    }

    #[tokio::test]
    async fn test_resolve_address_accepts_literals() {
        assert_eq!(
            resolve_address("127.0.0.1", 80).await.unwrap(),
            vec![addr("127.0.0.1:80")]
        );
        assert_eq!(
            resolve_address("::1", 80).await.unwrap(),
            vec![addr("[::1]:80")]
        );
        assert_eq!(
            resolve_address("[::1]", 80).await.unwrap(),
            vec![addr("[::1]:80")]
        );
        assert_eq!(
            resolve_address("::ffff:127.0.0.1", 80).await.unwrap(),
            vec![addr("127.0.0.1:80")]
        );
    }

    #[tokio::test]
    async fn test_connect_skips_unreachable_candidate_quickly() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let reachable = listener.local_addr().unwrap();

        // 100::/64 is the RFC 6666 discard prefix: packets to it are never answered.
        let black_hole = SocketAddr::new(IpAddr::V6(Ipv6Addr::new(0x100, 0, 0, 0, 0, 0, 0, 1)), 9);

        let started = Instant::now();
        let (_stream, connected) =
            connect_to_candidates::<TcpStream>(vec![black_hole, reachable], ())
                .await
                .unwrap();

        assert_eq!(connected, reachable);
        assert!(started.elapsed() < Duration::from_secs(5));
    }

    #[tokio::test]
    async fn test_connect_reports_error_when_all_candidates_fail() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let closed = listener.local_addr().unwrap();
        drop(listener);

        let result = connect_to_candidates::<TcpStream>(vec![closed], ()).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_connect_fails_without_candidates() {
        assert!(
            connect_to_candidates::<TcpStream>(vec![], ())
                .await
                .is_err()
        );
    }
}
