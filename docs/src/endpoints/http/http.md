# HTTP endpoint

HTTP endpoint is a listening point where the Tunnelize server listens for incoming HTTP requests. It allows clients to tunnel local HTTP traffic through the Tunnelize server. 

<img src="../../diagrams/httpendpoint.mermaid.svg" alt="Http tunnel explanation">

Tunnels configured to forward HTTP traffic first connect to the server where they get
assigned a domain to where a client can connect to through a browser to access the local
HTTP server.

When a client first connects to the HTTP endpoint, server uses the `Host` header
to decide to which tunnel it needs to connect to. After tunnel is found, a link is
established between client and tunnel and data is forwarded until either side closes the
connection.

## Configuring endpoint

Default HTTP endpoint configuration looks like this:

```json
{
    "type": "http",
    "port": 3457,
    "encryption": {
        "type": "none"
    },
    "address": null,
    "max_client_input_wait_secs": 10,
    "hostname_template": "tunnel-{name}.localhost",
    "full_url_template": null,
    "allow_custom_hostnames": true,
    "require_authorization": null
}
```

Fields:

| Field                      | Description                                                                                                                  | Default Value                    |
| -------------------------- | ---------------------------------------------------------------------------------------------------------------------------- | -------------------------------- |
| type                       | The type of the connection. Always `http` for http endpoint.                                                                 | No default                       |
| port                       | The port number for the connection                                                                                           | No default                       |
| encryption                 | The type of encryption used to enable HTTPS. See [configuring encryption](#configuring-encryption) below.                    | No default                       |
| address                    | The address for the connection to bind to. Defaults to 0.0.0.0 (all interfaces).                                             | 0.0.0.0                          |
| max_client_input_wait_secs | Maximum amount of seconds on how long to wait between start of TCP connection and first request being sent.                  | 10                               |
| hostname_template          | Template for the hostname to use when generating a hostname. See [configuring templates](#configuring-templates) below.      | No default                       |
| full_url_template          | Template for the full URL to use when returning it to the tunnel. See [configuring templates](#configuring-templates) below. | Automatic generation if not set. |
| allow_custom_hostnames     | Whether custom hostnames are allowed. See [configuring templates](#configuring-templates) below.                             | No default                       |
| require_authorization      | Whether authorization is required. See [configuring authorization](#configuring-authorization) below.                        | No authorization required        |

### Configuring encryption

When encryption is set, HTTP endpoint will use HTTPS protocol to tunnel data, even if your local server is not using
HTTPS. This is useful for securely sending data across the web and also to properly test how your local server would
behave in HTTPS environment.

See [setting up certificates](../../setting-up-certificates.md) for information on how to setup certificates for
server and tunnel.

There are two ways of setting the encryption, using a custom certificate or a servers own certificate.

**Using a server's own certificate**

This is the simpler approach as it will use the Tunnelize server's already predefined certificate defined in
server's configuration. This allows you to use it directly for HTTP endpoint without the need to specify it
multiple times.

> **Important**  
> 
> If tunnelize server is not using encryption when tunneling data but HTTP endpoint requires it, this will result
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

Using a custom certificate allows you to set a custom TLS certificate for HTTPS which may be different from 
tunnelize server's own certificate. This allows you to create multiple HTTP endpoints, each with its own 
certificate.

Configuration will look like:


### Configuring templates

For HTTP endpoints you can set templates to define how an URL will will be generated for a tunnel. There are two templates
you can set: 


```json
{
    // ...other fields
    "hostname_template": "tunnel-{name}.localhost",
    "full_url_template": "http://{hostname}:{port}",
}
```

Setting `hostname_template` is required and that template will let HTTP endpoint generate a random name for the tunnel
in the `{name}` part or use a custom name if `allow_custom_hostnames` is set.

When using `allow_custom_hostnames` name defined, `desired_name` defined in the [tunnel proxy configuration](../../setting-up-tunnel.md#setting-up-http) for
HTTP tunnel will be used unless it is already taken. If its already taken, a similar name will be autogenerated.

Setting `full_url_template` is useful if you are using tunnelize server behind something like an nginx or Apache, where
the HTTP endpoint port is not the same for client as it is the same for nginx or Apache. This will allow you to set the
template you wish server to return to the tunnel after registration.
Parameter `{hostname}` will be replaced by name generated by `hostname_template` and `{port}` with the port in HTTP
endpoint (or you can omit this or set your own port if needed).

### Configuring authorization

If you do not wish everyone to see your local tunnel while it is running, you can set an authorization where user needs
to enter an username and password in the browser to access your tunnel.

Set this configuration for that:

```json
{
    "require_authorization": {
        "realm": "exampleRealm",
        "username": "user123",
        "password": "pass123",
    }
}
```

Here is the breakdown of the parameters:

| Parameter | Description                                                                                                                             | Example Value  |
| --------- | --------------------------------------------------------------------------------------------------------------------------------------- | -------------- |
| realm     | A string that specifies the protection space. It is used to define the scope of protection for the browser. This field is not required. | "exampleRealm" |
| username  | The username required for authentication.                                                                                               | "user123"      |
| password  | The password required for authentication.                                                                                               | "pass123"      |


# Working with existing HTTP server

If you are using a http server like Apache or nginx it is possible to make tunnelize work with it. See links below
for your http server:

* [Working with nginx](./working-with-nginx.md)
* [Working with Apache](./working-with-apache.md)