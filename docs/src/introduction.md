# Introduction

Tunnelize is a self-hosted tunnel server and client written in Rust. It provides secure tunneling for HTTP, TCP, and UDP traffic, along with a monitoring API for managing connections. Tunnelize allows users to create secure tunnels between two endpoints, ensuring that data transmitted over the network is encrypted and protected. It supports various protocols and offers features such as secure forwarding, monitoring via CLI, and client provisioning from a server.

# Main Features

* **HTTP Tunneling**: Securely tunnel HTTP requests and expose the local server to the internet
* **TCP and UDP Tunneling**: Tunnel local TCP and UDP traffic via ports
* **Client Provisioning**: Provision tunnel configuration from connecting to the server and
downloading the configuration.
* **Secure Forwarding**: Local traffic can be transferred via TLS to the server.
* **Monitoring**: Manage and monitor tunnels conveniently via API or CLI commands.