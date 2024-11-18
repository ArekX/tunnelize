# Setting up endpoint encryption

Encryption is available for [HTTP](./http/http.md), [TCP](./tcp.md) and [Monitoring](./monitoring.md) endpoints.

See [setting up certificates](../../setting-up-certificates.md) for information on how to setup certificates for
server and tunnel.

There are two ways of setting the encryption, using a custom certificate or a servers own certificate.

**Using main server's certificate**

This is the simpler approach as it will use the Main tunnelize server's already predefined certificate defined in
server's configuration. This allows you to use it directly for this endpoint without the need to specify it
multiple times.

> **Important**  
> 
> If tunnelize server is not using encryption when tunneling data but this endpoint requires it, this will result
> in an error and server will not be able to run properly.


Configuration will look like:

```json
{
    // ...other fields
    "encryption": {
        "type": "tls",
    },
}
```

**Using a custom certificate**

Using a custom certificate allows you to set a custom TLS certificate for an endpoint which may be different from 
tunnelize server's own certificate. This allows you to create multiple endpoints, each with its own 
certificate.

This is useful if you are serving something like HTTP endpoints which have a different wildcard certificate from
the main server which is not using a wildcard.

Configuration will look like:

```json
{
    "encryption": {
        "type": "tls",
        "cert_path": "/path/to/server.crt", 
        "key_path": "/path/to/server.key" 
    }
}
```
