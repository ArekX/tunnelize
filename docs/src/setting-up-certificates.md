# Setting up certificates

Certificates are needed in order for you to use encrypted connections in tunnelize. Certificates can be self-signed
and using a certificate authority like Let's Encrypt.

## Self-signed certificates

Self-signed certificates are SSL/TLS certificates that are signed by the same entity whose identity they certify. 
Unlike certificates issued by a trusted certificate authority (CA), self-signed certificates are not automatically 
trusted by browsers and operating systems. They are typically used for testing, development, or internal purposes 
where trust can be manually established.

In order to generate self signed certificates we will use `openssl` command.

### Generating Certificate Authority (CA)

First step is to generate a certificate authority. 

```bash
openssl genrsa -out ca.key 4096
openssl req -new -x509 -days 365 -key ca.key -out ca.crt \
    -subj "/C=US/ST=State/L=City/O=YourCA/CN=localhost"
```

This will generate a `ca.crt` file which you will will use by the tunnel to validate the server certificate.


> Certificates have an expiry time. In this example expiration is set to 1 year.
 
Make sure you replace `/C=US/ST=State/L=City/O=YourCA/CN=localhost` with proper values for your certificate, in this
case a dummy localhost certificate will be created.

Here's a breakdown of those OpenSSL Distinguished Name (DN) parameters used in certificate generation:

| Parameter | Name         | Description                        | Example                      |
| --------- | ------------ | ---------------------------------- | ---------------------------- |
| `/C`      | Country      | Two-letter country code            | `US` for United States       |
| `/ST`     | State        | State or province name             | `California`                 |
| `/L`      | Locality     | City or locality name              | `San Francisco`              |
| `/O`      | Organization | Organization or company name       | `Example Corp`               |
| `/CN`     | Common Name  | Fully qualified domain name (FQDN) | `localhost` or `example.com` |

You can also import this certificate authority file into your operating system or a browser to make it a trusted certificate.

### Generating a server certificate

Next step is to generate a server certificate. Before we can do that we need to setup a `server.conf` configuration
file for the certificate. 

Here is an example file for the configuration:

```ini
[req]
distinguished_name = req_distinguished_name
req_extensions = v3_req
prompt = no

[req_distinguished_name]
C = US
ST = State
L = City
O = Organization
CN = localhost

[v3_req]
basicConstraints = CA:FALSE
keyUsage = nonRepudiation, digitalSignature, keyEncipherment
subjectAltName = @alt_names

[alt_names]
DNS.1 = localhost 
IP.1 = 127.0.0.1
```

Here is aa breakdown of the configuration file:

| Section                    | Purpose                   | Description                                          |
| -------------------------- | ------------------------- | ---------------------------------------------------- |
| `[req]`                    | Request Settings          | Main configuration section for certificate requests  |
| `[req_distinguished_name]` | DN Information            | Contains the certificate subject information         |
| `[v3_req]`                 | X509v3 Extensions         | Defines certificate capabilities and constraints     |
| `[alt_names]`              | Subject Alternative Names | Defines additional hostnames/IPs for the certificate |

Let's look at each section in detail:

#### [req] Section
```
distinguished_name = req_distinguished_name   # Points to DN section
req_extensions = v3_req                      # Points to extensions section
prompt = no                                  # Don't prompt for values interactively
```

#### [req_distinguished_name] Section
```
C = US              # Country
ST = State          # State/Province
L = City            # Locality/City
O = Organization    # Organization
CN = localhost      # Common Name
```

#### [v3_req] Section
```
basicConstraints = CA:FALSE                              # Not a Certificate Authority
keyUsage = nonRepudiation, digitalSignature, keyEncipherment    # Allowed key uses
subjectAltName = @alt_names                             # Points to alt names section
```

#### [alt_names] Section
```
DNS.1 = localhost       # DNS name the cert is valid for
IP.1 = 127.0.0.1       # IP address the cert is valid for
```

Make sure that you set all valid DNS names and IP addresses for where you want to use your server.

### Signing server certificate

Current server certificate cannot be used with your CA because it is not yet signed. To sign it run the following
command:

```bash
openssl x509 -req -days 825 -in server.csr \
    -CA ca.crt -CAkey ca.key -CAcreateserial \
    -out server.crt \
    -extfile server.conf -extensions v3_req
```

Note that the same `server.conf` is used for signing.

After signing you will be able to use `server.crt` and `server.key` in your tunnelize server by setting it in the
encryption part:

```json
{
    "server": {
        // ... other fields
        "encryption": {
            "type": "tls",
            "cert_path": "/path/to/server.crt", 
            "key_path": "/path/to/server.key" 
        }
    }
}
```

Your tunnel encryption will need to be pointed to `ca.crt` file:


```json
{
    "tunnel": {
        // ... other fields
        "encryption": {
            "type": "tls",
            "cert": "/path/to/ca.crt"
        }
    }
}
```

## Setting up certificates using Let's Encrypt

Before starting, make sure that you **have access to DNS zone for your domain and can change it**.

First, install Certbot:
```bash
# For Ubuntu/Debian
sudo apt update
sudo apt install certbot

# For CentOS/RHEL
sudo dnf install epel-release
sudo dnf install certbot
```

Generate the wildcard certificate:
```bash
sudo certbot certonly --manual --preferred-challenges=dns --server https://acme-v02.api.letsencrypt.org/directory -d *.your-hostname.com
```

Replace `your-hostname.com` with your domain.

Wait until you get propmpted by Certbot:
1. You'll receive a TXT record value
2. Create a DNS TXT record at your domain dns zone:
   - Name/Host: `_acme-challenge` 
   - Type: TXT
   - Value: The string provided by Certbot
   - TTL: Use lowest possible value (e.g., 60 seconds)

Once verified, press Enter in the Certbot prompt to complete the process.

Your certificates will be stored at (based on your domain name):
- Private key: `/etc/letsencrypt/live/your-hostname.com/privkey.pem`
- Certificate: `/etc/letsencrypt/live/your-hostname.com/fullchain.pem`

> Important
> 
> Let's encrypt certificate usually lasts for 90 days, after which you will need to renew it by running the same generate
> command above and following the process. If your DNS zone has an API, you could automate this process with Certbot
> [DNS plugins](https://eff-certbot.readthedocs.io/en/stable/using.html#dns-plugins).

Server will need to be configured as: 

```json
{
    "server": {
        // ... other fields
        "encryption": {
            "type": "tls",
            "cert_path": "/etc/letsencrypt/live/your-hostname.com/fullchain.pem", 
            "key_path": "/etc/letsencrypt/live/your-hostname.com/privkey.pem" 
        }
    }
}
```

Tunnel will need to be configured as:
```json
{
    "server": {
        // ... other fields
        "encryption": {
            "type": "native-tls",
        }
    }
}
```

In this case `native-tls` is used to use your OS certificates because Let's encrypt Certificate Authority (CA) is
normally trusted by your operating system.