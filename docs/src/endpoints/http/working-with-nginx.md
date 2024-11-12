# Working with nginx

If you are using nginx in your server it is possible to setup a tunnelize server to work together with nginx. In this
case tunnelize server will use a HTTP endpoint and it will be proxied through the nginx server to the user.

## Configuration without SSL

<div class="warning">
Make sure your DNS support [wildcard domains](https://en.wikipedia.org/wiki/Wildcard_DNS_record).
</div>

Configure your HTTP endpoint similar to this:

```json
{
    "type": "http",
    "port": 3457,
    "encryption": {
        "type": "none"
    },
    "max_client_input_wait_secs": 10,
    "hostname_template": "tunnel-{name}.your-hostname.com",
    "allow_custom_hostnames": true,
}
```

Then create a virtual host in nginx like this:

```nginx
server {
    listen 80;
    server_name ~^tunnel-(?<subdomain>\w+)\.your-hostname\.com$; # Set prefixed subdomain so that you can allow for any kind of tunnels

    # Increase the client request timeout
    client_body_timeout 60s;
    client_header_timeout 60s;

    # Increase proxy timeouts for connecting to the backend
    proxy_connect_timeout 60s;
    proxy_send_timeout 60s;
    proxy_read_timeout 60s;

    # Keep connections alive for a longer time
    keepalive_timeout 65s;

    location / {
        proxy_pass http://0.0.0.0:3457; # Set port to tunnelize server

        # This is required for tunnelize to figure out where to route to.
        proxy_set_header Host $host;

        # Pass WebSocket headers only when the connection is upgrading
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection $connection_upgrade;

        # Other proxy settings (optional)
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        proxy_buffering off;

        proxy_max_temp_file_size 0;
    }
}

# This is mapping for websocket support
map $http_upgrade $connection_upgrade {
    default "close";
    websocket "upgrade";
}
```

## Configuration with SSL

<div class="warning">
1. Make sure your DNS support [wildcard domains](https://en.wikipedia.org/wiki/Wildcard_DNS_record).
2. Make sure that you have a certificate 
</div>