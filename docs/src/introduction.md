# Introduction

Tunnelize is a self-hosted tunnel and server. It allows users to create secure tunnels between two endpoints, ensuring 
that data transmitted over the network is encrypted and protected. It supports forwarding HTTP, TCP and UDP traffic 
and supports encrypted connections, monitoring via CLI, and more.

## How this works

<image src="./diagrams/intro.mermaid.svg" alt="Tunnelize diagram">

When you set up a tunnelize server, based on the configuration, server will create one or more endpoints. These 
endpoints serve as entry points (HTTP, TCP, UDP) for anyone you are trying communicate with (client). These clients will connect to the endpoint
itself and tunnelize server will find the available tunnel for that endpoint (for example on TCP it is based on port,
for HTTP its based on Host header, etc.).

If there is an available tunnel, tunelize will send a request to create a link session between that tunnel and the
client. Once the link is estabilshed, data will be forwarded between both parties until one party closes the
connection.

# Features

* **Traffic tunneling**: Tunnel local traffic to HTTP/HTTPS, TCP and UDP endpoints
* **Secure Connection**: Securely connect to the tunnel server and allow secure connections on endpoints
* **Client Provisioning**: Initialize client by downloading settings from tunnel server.
* **Monitoring**: View and manage active tunnels, clients and links via CLI or JSON API

# Quickstart

Download tunnelize for your system from [releases page](https://github.com/ArekX/tunnelize/releases/latest).

Then initalize configuration by running `tunnelize init`. This will create a `tunnelize.json` with default tunnel
and server configuration.

Run a local HTTP server on port 8080. This will be the server we forward traffic from.

Run `tunnelize server`. This will run the server at port 3456 (by default) for the main server, creating listeners for all default 
endpoints (the default HTTP endpoint at 3457).

Run `tunnelize tunnel`. This will connect to the server at port 3456 and tunnel traffic from your local server. In the response you will see the URL assigned for you
to tunnel from, assuming default config, it will be something like:

```
[Forward|http] localhost:8080 -> http://tunnel-myname.localhost:3457
```

Open a browser and connect to [http://tunnel-myname.localhost:3457](http://tunnel-myname.localhost:3457) (this should work as expected in modern browsers like Chrome and Firefox) and you be able see the results from your local server at 8080.

See other topics for setup information:
* [Setting up a server](./setting-up-server.md)
* [Setting up a tunnel](./setting-up-tunnel.md)
* [Monitoring](./monitoring.md)
* [Commands reference](./commands.md)