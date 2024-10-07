use std::net::ToSocketAddrs;

use tokio::io::Result;

pub fn resolve_hostname(hostname: &String) -> Result<std::net::SocketAddr> {
    let addreses = hostname.to_socket_addrs()?;

    for addr in addreses {
        if addr.is_ipv4() {
            return Ok(addr);
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        "Address is not IPv4",
    ))
}
