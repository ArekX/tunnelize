# Setting up a tunnel

Tunneling is the main purpose of tunnelize. It will allow you to tunnel any kind of local data packets from your local 
through the tunnelize server of your choice up to the desired client.

# Initialization

To start tunneling, first initialize tunnel configuration. This can be done in two ways:

## Initalizing using default config

This will create `tunnelize.json` configuration file with default configuration you can use to setup your tunnels.

To do this run `tunnelize init tunnel`

Keep in mind that this requires you to already know the proper tunnelize server configuration.

## Provisioning via server config

Tunnelize is able to connect to the server directly, pull in all correct configuration and create an example
tunnel configuration you can directly use without having to have a full knowledge of the tunnelize server.

To do this run:

```sh
tunnelize init tunnel --server=my-tunnelize-server.com
```

Tunnelize will connect to the `my-tunnelize-server.com` at default port 3456, download information and create a config you can use to 
forward your local connections.  If your server is using another port add it via :PORT (for example: `my-tunnelize-server.com:5050`).

Use following options to handle other cases:

<table>
  <thead>
    <tr>
      <th style="width: 20%;">Option</th>
      <th style="width: 50%;">Description</th>
      <th style="width: 30%;">Example</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>--key</td>
      <td>Specifies the tunnel key to use for authenticating with the server.</td>
      <td><code>--key=my-tunnel-key</code></td>
    </tr>
    <tr>
      <td>--tls</td>
      <td>Enables TLS for the connection to the server.</td>
      <td><code>--tls</code></td>
    </tr>
    <tr>
      <td>--ca</td>
      <td>Path to the custom CA (Certificate Authority) certificate file for TLS. If not specified, it will use CA certificates in current OS.</td>
      <td><code>--cert=/path/to/ca.crt</code></td>
    </tr>
  </tbody>
</table>

# Configuring a tunnel manually

To configure the tunnel manually, create a `tuhnelize.json` and configure it:

```json
{
  "tunnel": {
    "name": "my-tunnel",
    "server_address": "localhost",
    "proxies": [
      // ...proxy configuration
    ]
  }  
}
```

Fields:

| Name                               | Description                                                                                      | Default Value    |
| ---------------------------------- | ------------------------------------------------------------------------------------------------ | ---------------- |
| name                               | Name of the tunnel. Optional, helps identify the tunnel in monitoring.                           | Empty string     |
| server_address                     | Hostname or address to the main tunnelize server.                                                | No default       |
| server_port                        | Port of the server                                                                               | 3456             |
| forward_connection_timeout_seconds | How much time to wait in seconds for first response from your local server before disconnecting. | 30               |
| encryption                         | Type of encryption. **See** [configuring encryption](#configuring-encryption) below.             | No encryption    |
| tunnel_key                         | Key for the tunnel                                                                               | No key specified |
| monitor_key                        | Key for monitoring                                                                               | No key specified |
| proxies                            | Proxy configuration. See [configuring proxies](#configuring-proxies) below.                      | No default       |

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
    "type": "tls"
}
```
TLS encryption will be used.

All available fields are:
| Name    | Value                                                                                                                                                      | Default                   |
| ------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------- |
| type    | Type of encryption. Always `tls` in this case.                                                                                                             | No default                |
| ca_path | Path to certificate authority (`ca.crt`) certificate for self-signed certificate validation. If not set, OS native certificates will be used for checking. | No certificate authority. |

See [setting up certificates](./setting-up-certificates.md) for information on how to use certificate files.


# Configuring proxies

Proxies decide what kind of traffic you want to tunnel. Keep in mind that if tunnelize server is not configured to
tunnel a specific endpoint, you will not be able to tunnel that kind of traffic. Another thing to note, you will need
to know names of the endpoints specified in the server in order to use them. If you are hosting the server yourself, 
that is easy to find out, but if you are using someone else's server, that might be a challenge, in which case you
should [provision the configuration](#provisioning-via-server-config) via the server.

To setup a proxy, add a new value in proxies array:

```json
{
  "tunnel": {
    "proxies": [
       {
        "endpoint_name": "http",
        "address": "localhost",
        "port": 8080,
        "endpoint_config": {
           // proxy specific endpoint settings
        }
      }
    ]
  }  
}
```

Fields:
| Name            | Description                                                                               | Default Value |
| --------------- | ----------------------------------------------------------------------------------------- | ------------- |
| endpoint_name   | The name of the endpoint of the same type this proxy will forward connections to.         | No default    |
| address         | The IP address of the server you want to forward connection from.                         | No default    |
| port            | The port number of server you want to forward connection from.                            | No default    |
| endpoint_config | Proxy settings to pass to the endpoint. Must be valid values for the endpoint. See below. | No default    |

> Important
>
> When defining an endpoint for a proxy, you must make sure that type of the proxy matches the type of the endpoint
> otherwise, your tunnel connection will be rejected.

You can set up following proxies:
* HTTP
* TCP
* UDP

## Setting up HTTP

To setup endpoint config for HTTP, set the following JSON on the HTTP proxy:

```json
{
  "tunnel": {
    // ...other fields
    "proxies": [
      {
         // ...other fields for http proxy
         "endpoint_config": {
            "type": "http",
            "desired_name": "desired-name"
         }
      }
    ]
  }
}
```

Fields:
| Name         | Description                                                                                                                                                                                                                       | Default Value |
| ------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------- |
| type         | Type of tunnel. For HTTP endpoint, always http.                                                                                                                                                                                   | No default    |
| desired_name | Desired name, will be used in `{name}` part in endpoint [hostname template](./endpoints/http/http.md#configuring-templates) which will be assigned to this proxy if allowed and not already taken. Otherwise, it will be ignored. | No value      |

## Setting up TCP

To setup endpoint config for HTTP, set the following JSON on the HTTP proxy:

```json
{
  "tunnel": {
    // ...other fields
    "proxies": [
      {
         // ...other fields for http proxy
         "endpoint_config": {
            "type": "tcp",
            "desired_port": 1234
         }
      }
    ]
  }
}
```

Fields:
| Name         | Description                                                                                                        | Default Value |
| ------------ | ------------------------------------------------------------------------------------------------------------------ | ------------- |
| type         | Type of tunnel. For TCP endpoint, always tcp.                                                                      | No default    |
| desired_port | Desired port which will be assigned to this proxy if allowed and not already taken. Otherwise, it will be ignored. | No value      |

## Setting up UDP


To setup endpoint config for HTTP, set the following JSON on the HTTP proxy:

```json
{
  "tunnel": {
    // ...other fields
    "proxies": [
      {
         // ...other fields for http proxy
         "endpoint_config": {
            "type": "udp",
            "desired_port": 1234,
            "bind_address": "0.0.0.0:0"
         }
      }
    ]
  }
}
```

Fields:
| Name         | Description                                                                                                                                                  | Default Value |
| ------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------- |
| type         | Type of tunnel. For UDP endpoint, always udp.                                                                                                                | No default    |
| desired_port | Desired port which will be assigned to this proxy if allowed and not already taken. Otherwise, it will be ignored.                                           | No value      |
| bind_address | Bind address and port which will be used to listen to the data from your local UDP server. If not set, random available port on addres 0.0.0.0 will be used. | 0.0.0.0:0     |