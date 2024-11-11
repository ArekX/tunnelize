# Tunneling

Tunneling is the main purpose of tunnelize. It will allow you to tunnel any kind of local data packets from your local 
through the tunnelize server of your choice up to the desired client.

# Initialization

To start tunneling, first initialize tunnel configuration. This can be done in two ways:

## Initalizing using default config

This will create `tunnelize.json` configuration file with default configuration you can use to setup your tunnels.

To do this run `tunnelize init tunnel`

Keep in mind that this requires you to already know the proper tunnelize server configuration.

## Provisioning via server config

Tunnelize is able to connect to the server directly, pull in all correct configuration and create an example
tunnel configuration you can directly use without having to have a full knowledge of the tunnelize server.

To do this run:

```sh
tunnelize init tunnel --server=my-tunnelize-server.com
```

Tunnelize will connect to the `my-tunnelize-server.com` at default port 3456, download information and create a config you can use to 
forward your local connections.  If your server is using another port add it via :PORT (for example: `my-tunnelize-server.com:5050`).

Use following options to handle other cases:

<table>
  <thead>
    <tr>
      <th style="width: 20%;">Option</th>
      <th style="width: 50%;">Description</th>
      <th style="width: 30%;">Example</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>--key</td>
      <td>Specifies the tunnel key to use for authenticating with the server.</td>
      <td><code>--key=my-tunnel-key</code></td>
    </tr>
    <tr>
      <td>--tls</td>
      <td>Enables TLS for the connection to the server.</td>
      <td><code>--tls</code></td>
    </tr>
    <tr>
      <td>--cert</td>
      <td>Path to the custom CA (Certificate Authority) certificate file for TLS. If not specified, it is assumed the server is using a non self-signed certificate (<code>native-tls</code>) meaning it will use CA predefined in OS.</td>
      <td><code>--cert=/path/to/ca.crt</code></td>
    </tr>
  </tbody>
</table>