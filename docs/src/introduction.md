# Introduction

Tunnelize is a self-hosted tunnel and server. It allows users to create secure tunnels between two endpoints, ensuring 
that data transmitted over the network is encrypted and protected. It supports forwarding HTTP, TCP and UDP traffic 
and supports encrypted connections, monitoring via CLI, and more.

## How this works

<image src="./diagrams/intro.mermaid.svg" alt="Tunnelize diagram">

You host a tunnelize server on your server, like a VPS or an instance somewhere in the cloud. This will be the main server you forward your local traffic to, then you configure all
of the endpoints you wish that your server has and then you run the tunnel command to forward
that traffic to the server.

Server will manage the access checks for your tunnel and the clients and all traffic routing.

# Main Features

* **Traffic tunneling**: Tunnel local traffic to HTTP/HTTPS, TCP and UDP endpoints
* **Secure Connection**: Securely connect to the tunnel server and allow secure connections on endpoints
* **Client Provisioning**: Initialize client by downloading settings from tunnel server.
* **Monitoring**: View and manage active tunnels, clients and links.

# Quickstart

First initalize configuration by running `tunnelize init`. This will create a `tunnelize.json` with default tunnel
and server configuration.

Run a local HTTP server on port 8080. This will be the server we forward traffic from.

Run `tunnelize server`. This will run the server at port 3456 for tunnelize, creating listeners for all default 
endpoints including the default HTTP endpoint at 3457.

Run `tunnelize tunnel`. This will connect to the server at port 3456 and tunnel traffic from your local server.

Open a browser and connect to [http://localhost:3457](http://localhost:3457) and you be able see the results from your local server at 8080.

See other topics for setup information:
* [Setting up a server](./setting-up-server.md)
* [Setting up a tunnel](./setting-up-tunnel.md)
* [Monitoring](./monitoring.md)
* [Commands reference](./commands.md)