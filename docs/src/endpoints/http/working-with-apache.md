# Working with Apache

If you are using Apache in your server, it is possible to setup a tunnelize server to work together with Apache. 
In this case, tunnelize server will use a HTTP endpoint and it will be proxied through the Apache server to the user.

## Configuration without SSL

> **Important**  
> 
> Make sure your DNS supports [wildcard domains](https://en.wikipedia.org/wiki/Wildcard_DNS_record).

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

Then create a virtual host configuration in Apache like this:

```apache
# Enable required modules
LoadModule proxy_module modules/mod_proxy.so
LoadModule proxy_http_module modules/mod_proxy_http.so
LoadModule proxy_wstunnel_module modules/mod_proxy_wstunnel.so
LoadModule rewrite_module modules/mod_rewrite.so

# Virtual Host configuration
<VirtualHost *:80>
    # Use wildcard ServerName to match tunnel subdomains
    ServerName tunnel-prefix.your-hostname.com
    ServerAlias tunnel-*.your-hostname.com

    # Set longer timeouts
    TimeOut 60
    ProxyTimeout 60
    
    # Enable WebSocket proxy
    RewriteEngine On
    RewriteCond %{HTTP:Upgrade} websocket [NC]
    RewriteCond %{HTTP:Connection} upgrade [NC]
    RewriteRule ^/?(.*) "ws://localhost:3457/$1" [P,L]

    # Proxy configuration
    ProxyPass / http://localhost:3457/
    ProxyPassReverse / http://localhost:3457/
    
    # Pass required headers
    ProxyPreserveHost On
    RequestHeader set X-Forwarded-Proto "http"
    RequestHeader set X-Real-IP %{REMOTE_ADDR}s
    RequestHeader set X-Forwarded-For %{REMOTE_ADDR}s

    # Disable response buffering
    SetEnv force-proxy-request-1.0 1
    SetEnv proxy-nokeepalive 1
</VirtualHost>
```

## Configuration with SSL

> **Important**  
> Make sure your DNS supports [wildcard domains](https://en.wikipedia.org/wiki/Wildcard_DNS_record). Also make sure that you have a wildcard certificate setup.

Use the same configuration as above, but modify the VirtualHost configuration to include SSL:

```apache
<VirtualHost *:443>
    # Same ServerName and ServerAlias as above
    ServerName tunnel-prefix.your-hostname.com
    ServerAlias tunnel-*.your-hostname.com

    # SSL Configuration
    SSLEngine on
    SSLCertificateFile /etc/letsencrypt/live/example.com-0001/fullchain.pem
    SSLCertificateKeyFile /etc/letsencrypt/live/example.com-0001/privkey.pem
    Include /etc/letsencrypt/options-ssl-apache.conf

    # All other configuration remains the same as the non-SSL version
    TimeOut 60
    ProxyTimeout 60
    
    RewriteEngine On
    RewriteCond %{HTTP:Upgrade} websocket [NC]
    RewriteCond %{HTTP:Connection} upgrade [NC]
    RewriteRule ^/?(.*) "wss://localhost:3457/$1" [P,L]

    ProxyPass / http://localhost:3457/
    ProxyPassReverse / http://localhost:3457/
    
    ProxyPreserveHost On
    RequestHeader set X-Forwarded-Proto "https"
    RequestHeader set X-Real-IP %{REMOTE_ADDR}s
    RequestHeader set X-Forwarded-For %{REMOTE_ADDR}s

    SetEnv force-proxy-request-1.0 1
    SetEnv proxy-nokeepalive 1
</VirtualHost>
```

Note the key differences from the nginx configuration:

1. Apache requires explicit module loading for proxy and WebSocket support
2. WebSocket proxying is handled through mod_rewrite rules rather than headers
3. The header setting syntax is different but achieves the same result
4. SSL configuration uses Apache's SSLEngine directives instead of nginx's ssl_ directives

Make sure all required Apache modules are enabled:
```bash
a2enmod proxy
a2enmod proxy_http
a2enmod proxy_wstunnel
a2enmod rewrite
a2enmod ssl  # If using SSL
```

After making these changes, restart Apache to apply the configuration:
```bash
sudo systemctl restart apache2
```