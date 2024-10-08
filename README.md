# tunnelize

Self-Hosted Tunnel Server and Client written in Rust.

**Note:** This application is still not in stable phase but it can be used to host a HTTP tunnel.

Supported Tunnels:
* HTTP

# Building

Run `cargo build --release` to build the release version of tunnelize, built application will be in
`target/release/tunnelize`.

# Running

To run the tunnel you first need to initialize `tunnelize.json` configuration so that tunnel knows
where to connect to. To do this run `tunnelize tunnel --init` to create it. You will see the configuration for both
server and tunnel so configure the part you wish to run. Below is a config for tunnel:

```js
{
  "tunnel": {
    "server_address": "example.com:3456", // make sure this port is open for tunnel to connect to
    "hostnames": [
      {
        "desired_name": "testapp", // this name will be used for {name} part (check below)
        "forward_address": "0.0.0.0:8000" // Port from your local to forward
      },
      {
        "forward_address": "0.0.0.0:3000" // this port will be forwarded to a generated name.
      }
    ],
    "auth_key": null // If you wish to set password authorization to use server, set the password here.
  }
}

```

# Hosting

## Setting up tunnelize server

To start hosting create configuration by running `tunnelize server --init` to create `tunnelize.json` configuration.

Then set following for your server:

```js
{
  "server": {
    "servers": [
      {
        "type": "http", // http server, currently only supported
        "client_port": 3457, // this is the port for nginx for proxying
        "tunnel_port": 3456, // this is the port for tunnels to connect to
        "auth_key": null, // set password for authorization
        "host_template": "tunnel-{name}.example", // set this to your domain, {name} part will be replaced by generated names
        "allow_custom_hostnames": true // set to true to allow tunnels to specify their own names for domains,
        "client_authorize_user": null // require client auth to use tunnel, set to  { "username": "admin", "password": "admin" } to define which credentials to use, add "realm": "name" to specify the realm for credentials
      }
    ]
  }
}

```

## Using nginx

## Without SSL

Make sure your DNS supports wildcard subdomains like `*.example.com`.

Add following configuration:

```
server {
    listen 80;
    server_name ~^tunnel-(?<subdomain>\w+)\.example\.com$; # Set prefixed subdomain so that you can allow for any kind of tunnels

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

## With SSL

Certbot is required and access to your DNS zone if you are using Let's Encrypt.

First run certbot with following settings (replace `example.com` with your domain):

```
sudo certbot certonly --manual --preferred-challenges=dns --server https://acme-v02.api.letsencrypt.org/directory -d *.example.com
```

You will get an acme challenge and set that as TXT record in your DNS zone then press enter to verify.

After verification check the path to your certificate and add following changes to nginx config defined above:

```
server {
    # ...
    listen 443 ssl; # change listen to this

    # Add SSL certificates 
    ssl_certificate /etc/letsencrypt/live/example.com-0001/fullchain.pem; # make sure this path matches to the certificate for certbot
    ssl_certificate_key /etc/letsencrypt/live/example.com-0001/privkey.pem; # make sure this path matches to the certificate for certbot
    include /etc/letsencrypt/options-ssl-nginx.conf; 
    ssl_dhparam /etc/letsencrypt/ssl-dhparams.pem;

    #...
}
```

Then restart nginx to apply changes and run `tunnelize server` to run the server.
