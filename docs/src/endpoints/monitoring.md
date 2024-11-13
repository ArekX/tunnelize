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
| type               | Type of service. Always monitoring for monitoring endpoint.                                  | No default    |
| port               | Port number                                                                                  | No default    |
| encryption         | Encryption for HTTPS access. See [configuring encryption](../configuring-encryption.md).     | No default    |
| address            | Service address. Defaults to 0.0.0.0 if not set.                                             | 0.0.0.0       |
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

# API endpoints