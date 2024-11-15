# Command reference

`tunnelize` command reference can be also shown by running `tunnelize help`.

| Command   | Subcommand          | Arguments               | Description                                                                                                                |
| --------- | ------------------- | ----------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| `init`    | `all`               | -                       | Initialize `tunnelize.json` for both tunnel and server with example configuration.                                         |
| `init`    | `tunnel`            | `-s, --server <SERVER>` | Initialize `tunnelize.json` for tunnel. If `-s, --server` is passed it will connect to tunnelize server to pull in config. |
|           |                     | `-t, --tls`             | Use TLS to connect to server                                                                                               |
|           |                     | `-c, --cert <CERT>`     | Path to custom CA certificate file for TLS                                                                                 |
|           |                     | `-k, --key <KEY>`       | Tunnel key for server authentication                                                                                       |
| `init`    | `server`            | -                       | Initialize `tunnelize.json` for server configuration                                                                       |
| `server`  |                     | `-c, --config <CONFIG>` | Starts tunnelize server using `tunnelize.json` from current directory.                                                     |
| `tunnel`  |                     | `-c, --config <CONFIG>` | Starts tunnelize tunnel using `tunnelize.json` from current directory.                                                     |
|           |                     | `-v, --verbose`         | Show detailed output for tunnel connection                                                                                 |
| `monitor` | `system-info`       | `-c, --config <CONFIG>` | Display system information.                                                                                                |
| `monitor` | `list-endpoints`    | `-c, --config <CONFIG>` | List all endpoints.                                                                                                        |
| `monitor` | `list-tunnels`      | `-c, --config <CONFIG>` | List all tunnels.                                                                                                          |
| `monitor` | `get-tunnel`        | `-c, --config <CONFIG>` | Get tunnel information by UUID.                                                                                            |
| `monitor` | `disconnect-tunnel` | `-c, --config <CONFIG>` | Disconnect tunnel by UUID.                                                                                                 |
| `monitor` | `list-clients`      | `-c, --config <CONFIG>` | List all clients.                                                                                                          |
| `monitor` | `get-client`        | `-c, --config <CONFIG>` | Get client information by UUID.                                                                                            |
| `monitor` | `list-links`        | `-c, --config <CONFIG>` | List all links.                                                                                                            |
| `monitor` | `get-link`          | `-c, --config <CONFIG>` | Get link information by UUID.                                                                                              |
| `monitor` | `disconnect-link`   | `-c, --config <CONFIG>` | Disconnect link by UUID.                                                                                                   |

On commands using `-c, --config`, if it is passed, it will load in that config json file, otherwise it will load `tunnelize.json` from current working directory.