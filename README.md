# tunnelize

> [!WARNING]  
> This project is still in active development and does not have a stable release. Expect bugs and things being changed all of the time.

Tunnelize is a self-hosted tunnel server and client written in Rust. It provides secure tunneling for HTTP, TCP, and UDP traffic, along with a monitoring API for managing connections.


## Features

- **HTTP Tunnel**: Tunnel HTTP requests by domains with HTTPS support.
- **TCP Tunnel**: Tunnel traffic via ports.
- **UDP Tunnel**: Tunnel traffic via ports.
- **Monitoring API**: Monitor and manage connections.
- **Secure Forwarding**: Tunnel client to server TLS connection.
- **Monitoring via CLI**: perform monitoring commands via CLI from the tunnel
- **Client provisioning**: Provision client settings from a server

## Building

To build the release version of Tunnelize, run:

```sh
cargo build --release
```

The built application will be located in `target/release/tunnelize`.