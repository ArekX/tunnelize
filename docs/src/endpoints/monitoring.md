# Monitoring endpoint

Monitoring endpoint is an API endpoint which allows the user to manage the tunnelize server. It exposes a JSON API
for managing tunnels, clients, links and monitoring system.

To setup a monitoring API configure endpoints like this:


```json
{
    "server":{
        // ...other fields
        "endpoints":{
            "monitoring-endpoint": {
                "type": "monitoring",
                "port": 3000,
                "encryption": {
                    "type": "none"
                },
                "address": null,
                "authentication": {
                    "type": "basic",
                    "username": "admin",
                    "password": "changethispassword"
                },
                "allow_cors_origins": {
                    "type": "any"
                }
            }
        }
    }
    
}
```

Fields:

| Key                | Description                                                                                  | Default Value |
| ------------------ | -------------------------------------------------------------------------------------------- | ------------- |
| type               | Type of service. Always `monitoring` for monitoring endpoint.                                | No default    |
| port               | Port number                                                                                  | No default    |
| encryption         | Encryption for HTTPS access. See [configuring encryption](./setting-up-encryption.md).       | No encryption |
| address            | Service address.                                                                             | 0.0.0.0       |
| authentication     | Type of authentication. See [configuring authentication](#configuring-authentication) below. | No default    |
| allow_cors_origins | CORS origins allowed.  See [configuring CORS](#configuring-cors) below.                      | any           |

# Configuring authentication

Authentication allows you to protect the monitoring endpoint from unauthorized acccess. It is important to set this on
production hosting to disallow outside access if you are using monitoring endpoint as the unauthorized user can manage
tunnel, client and link access.

There are two types of authorization you can set: basic and bearer.

Keep in mind that monitoring has bruteforce protection where user is kicked out for 5 minutes after 5 failed attempts.

**Setting up basic authorization**

Configuration will look like this:

```json
{
    "server":{
        // ...other fields
        "endpoints":{
            "monitoring-endpoint": {
                // ...other fields
                "authentication": {
                    "type": "basic",
                    "username": "admin",
                    "password": "changethispassword"
                },
            }
        }
    }
}
```

This will setup a basic authorization method where browser will ask you to enter this username and password to access
the endpoint.

**Setting up bearer authorization**

Bearer authorization is a more traditional token authorization as used in API requests. Your API client will send the
token in `Authorization: Bearer <token>` header and if the token value is correct, tunnelize will grant access.

Configuration looks like this:

```json
{
    "server":{
        // ...other fields
        "endpoints":{
            "monitoring-endpoint": {
                // ...other fields
                "authentication": {
                    "type": "bearer",
                    "token": "yourtoken",
                },
            }
        }
    }
}
```

# Configuring CORS

CORS (Cross-Origin Resource Sharing) allows you to control which origins are permitted to access resources on your 
server. This is important for security, especially if your monitoring endpoint is accessed from web applications hosted 
on different domains.

You can configure CORS in the `allow_cors_origins` field. There are three types of CORS configurations you can set: `any`, `none`, and `list`.

**Allow any origin**

This configuration allows any origin to access the monitoring endpoint.

```json
{
    "server":{
        // ...other fields
        "endpoints":{
            "monitoring-endpoint": {
                // ...other fields
                "allow_cors_origins": {
                    "type": "any"
                }
            }
        }
    }
}
```

**Disallow all origins**

This configuration disallows all origins from accessing the monitoring endpoint.

```json
{
    "server":{
        // ...other fields
        "endpoints":{
            "monitoring-endpoint": {
                // ...other fields
                "allow_cors_origins": {
                    "type": "none"
                }
            }
        }
    }
}
```

**Allow specific origins**

This configuration allows only specified origins to access the monitoring endpoint. You need to provide a list 
of allowed origins.

```json
{
    "server":{
        // ...other fields
        "endpoints":{
            "monitoring-endpoint": {
                // ...other fields
                "allow_cors_origins": {
                    "type": "list",
                    "origins": [
                        "https://example.com",
                        "https://anotherdomain.com"
                    ]
                }
            }
        }
    }
}
```

Make sure to configure CORS according to your security requirements to prevent unauthorized access from untrusted 
origins.

# API endpoints

| Endpoint                | Method | Description                                                           |
| ----------------------- | ------ | --------------------------------------------------------------------- |
| /system/info            | GET    | Retrieves system information including CPU usage, memory, and uptime. |
| /system/endpoints       | GET    | Lists all configured endpoints on the server.                         |
| /system/endpoints/:name | GET    | Retrieves information about a specific endpoint by name.              |
| /system/clients         | GET    | Lists all connected clients.                                          |
| /system/clients/:id     | GET    | Retrieves information about a specific client by ID.                  |
| /tunnels                | GET    | Lists all active tunnels.                                             |
| /tunnels/:id            | GET    | Retrieves information about a specific tunnel by ID.                  |
| /tunnels/:id            | DELETE | Disconnects a specific tunnel by ID.                                  |
| /links                  | GET    | Lists all active links.                                               |
| /links/:id              | GET    | Retrieves information about a specific link by ID.                    |
| /links/:id              | DELETE | Disconnects a specific link by ID.                                    |

