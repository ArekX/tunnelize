# TCP endpoint

TCP endpoint is a listener for TCP traffic. When this endpoint is started, it will listen to the client connections on
a specified port range. When a client connects to a specific port, server will look for a conected tunnel on that port
and if there is such a tunnel it will create a link between them and route data.

image

Configuration is setup as follows:

```json
{
    "server":{
        // ...other fields
        "endpoints":{
            "tcp": {
                "type": "tcp",
                "address": null,
                "allow_desired_port": true,
                "reserve_ports_from": 4000,
                "reserve_ports_to": 4050,
                "encryption": {
                    "type": "none"
                },
                "full_hostname_template": "localhost:{port}"
            }
        }
    }
    
}
```

Fields:

| Key                    | Description                                                                                           | Default Value |
| ---------------------- | ----------------------------------------------------------------------------------------------------- | ------------- |
| type                   | The type of the endpoint, in this case, always TCP.                                                   | No default    |
| address                | The address to bind to. If not set, 0.0.0.0 will be used, meaning all interfaces.                     | 0.0.0.0       |
| allow_desired_port     | Allows the use of a desired port if available. If not available, first available port will be chosen. | No default    |
| reserve_ports_from     | The starting port of the reserved range for this endpoint.                                            | No default    |
| reserve_ports_to       | The ending port of the reserved range range for this endpoint.                                        | No default    |
| encryption.type        | The type of encryption used. See [configuring encryption](#configuring-encryption) below.             | No default    |
| full_hostname_template | Template for the full hostname with port. See [configuring templates](#configuring-templates) below.  | No default    |

### Configuring encryption

When encryption is set, TCP will use TLS protocol to tunnel data, even if your local server is not using it. 

> Important
>
> If your TCP local server is already encrypted, it will not make sense to encrypt it again in this endpoint.

See [setting up certificates](../../setting-up-certificates.md) for information on how to setup certificates for
server and tunnel.

There are two ways of setting the encryption, using a custom certificate or a servers own certificate.

**Using a server's own certificate**

This is the simpler approach as it will use the Tunnelize server's already predefined certificate defined in
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

Using a custom certificate allows you to set a custom TLS certificate for TCP which may be different from 
tunnelize server's own certificate. This allows you to create multiple TCP endpoints, each with its own 
certificate.

Configuration will look like:


### Configuring templates

For TCP endpoints you can set templates to define how an URL will will be generated for a tunnel. There are two templates
you can set: 


```json
{
    "server":{
        // ...other fields
        "endpoints":{
            "tcp": {
                 // ...other fields
                "full_hostname_template": "localhost:{port}"
            }
        }
    }
    
}
```

Template you set here, will be returned by the server to the tunnel proxying the connection, to tell the user where
their local server can be reached from.

Placeholder `{port}` will be replaced by the port assigned to the tunnel.