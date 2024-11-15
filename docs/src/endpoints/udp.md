# UDP endpoint

UDP endpoint is a listener for UDP traffic. When this endpoint is started, it will listen to the client connections on
a specified port range. When a client connects to a specific port, server will look for a connected tunnel on that port
and if there is such a tunnel it will create a link between them and route data.

Configuration is setup as follows:

```json
{
    "server":{
        // ...other fields
        "endpoints":{
            "udp": {
                "type": "udp",
                "address": null,
                "allow_desired_port": true,
                "reserve_ports_from": 4000,
                "reserve_ports_to": 4050,
                "full_hostname_template": "localhost:{port}"
            }
        }
    }
    
}
```

Fields:

| Key                    | Description                                                                                           | Default Value |
| ---------------------- | ----------------------------------------------------------------------------------------------------- | ------------- |
| type                   | The type of the endpoint, in this case, always UDP.                                                   | No default    |
| address                | The address to bind to. If not set, 0.0.0.0 will be used, meaning all interfaces.                     | 0.0.0.0       |
| allow_desired_port     | Allows the use of a desired port if available. If not available, first available port will be chosen. | No default    |
| reserve_ports_from     | The starting port of the reserved range for this endpoint.                                            | No default    |
| reserve_ports_to       | The ending port of the reserved range range for this endpoint.                                        | No default    |
| full_hostname_template | Template for the full hostname with port. See [configuring templates](#configuring-templates) below.  | No default    |

### Configuring templates

For UDP endpoints you can set templates to define how an URL will will be generated for a tunnel. There are two templates
you can set: 


```json
{
    "server":{
        // ...other fields
        "endpoints":{
            "udp": {
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