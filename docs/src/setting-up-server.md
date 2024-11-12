# Setting up a server

To setup a server, first initialize the configuration by running `tunnelize server init`.

This will create initial default configuration in `tunnelize.json` for a server, see [Configuration](#configuring-the-server) for information about
specific attributes.

<div class="warning">
Default configuration is NOT SAFE for general use outside for testing. Make sure you properly go through configuration and setup your
tunnelize server.
</div>

Server will be run by just running `tunnelize` or `tunnelize server`, after which 
tunnelize server is ready to accept connections.

For information on how to setup a server as a service so that it keeps running even after OS restarts [see here](./setup-a-service.md).


# Configuring the server

Following is a typical default settings for a server:

```json
{
  "server": {
    "server_port": 3456,
    "server_address": null,
    "max_tunnel_input_wait": 30,
    "tunnel_key": null,
    "monitor_key": "changethismonitorkey",
    "endpoints": { /* ...endpoints... */ },
    "encryption": {
      "type": "none"
    },
    "max_tunnels": 50,
    "max_clients": 100,
    "max_proxies_per_tunnel": 10
  }
}
```

Fields:


| Field                    | Description                                                                                       | Default Value   |
| ------------------------ | ------------------------------------------------------------------------------------------------- | --------------- |
| `server_port`            | Port on which the server listens for tunnel connections.                                          | No default      |
| `server_address`         | Address to which the server will bind to. Defaults to 0.0.0.0 if not set or null.                 | 0.0.0.0         |
| `max_tunnel_input_wait`  | Maximum amount of time (in seconds) to wait from tunnel connection to first message from tunnel.  | No default      |
| `tunnel_key`             | Key which tunnel must have in order to be allowed to communicate.                                 | No key required |
| `monitor_key`            | Key which tunnelize tunnel must have in order to execute monitor commands on the server.          | No key required |
| `endpoints`              | Configuration for server endpoints. See [endpoints](#configuring-endpoints) for more information. | No default      |
| `encryption`             | TLS encryption settings. See [encryption](#configuring-encryption)                                | No default      |
| `max_tunnels`            | Maximum number of tunnels allowed on the server.                                                  | No default      |
| `max_clients`            | Maximum number of clients allowed on the server.                                                  | No default      |
| `max_proxies_per_tunnel` | Maximum number of proxies per tunnel allowed.                                                     | No default      |

## Configuring Encryption

It can be one of the two types:

**No encryption required**  
```json
{
    "type": "none"
}
```
In this case any tunnelize client can connect and pass data in unencrypted connection. This means
that all data passed between tunnel and server is visible to a third party.

**TLS encryption**
```json
{
    "type": "tls", 
    "cert_path": "/path/to/certificate/file", 
    "key_path": "/path/to/key/file"
}
```
Standard TLS encryption will be used. Keep in mind that in this case Tunnel must also use encryption with a certificate authority (if using self-signed) set or
with `native-tls` if you are using a known certificate authority like Let's Encrypt.

See [setting up certificates](./setting-up-certificates.md) for information on how to use certificate files.

## Configuring Endpoints

Endpoints are configured as follows:


```json
{
  "server": {
    // ... other fields
    "endpoints": { 
        "endpoint-name-1": { 
           "type": "http",
           // ...configuration for HTTP endpoint
        },
        "endpoint-name-2": { 
           "type": "tcp",
           // ...configuration for TCP endpoint
        },
        // ... other endpoints
     },
  }
}
```

You can create any number of endpoints to where clients can connect to your local servers. Each endpoint name
has to be unique as this is the name that your tunnel will send to identify to which endpoint it wants to proxy data
to.

There are multiple types of endpoints:
* [HTTP](./endpoints/http.md)
* [TCP](./endpoints/tcp.md)
* [UDP](./endpoints/udp.md)
* [Monitoring](./endpoints/monitoring.md)