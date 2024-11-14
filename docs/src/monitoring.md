# Monitoring

Monitoring in this project is designed to help you keep track of the system's performance and health. It allows you to 
observe various metrics and logs to ensure that everything is running smoothly and manage currently connected tunnels,
links and clients.

Monitoring can be done through an [api endpoint](./endpoints/monitoring.md) or through CLI commands. This section 
explains CLI commands.

## Configuration

Before you can run the monitoring commands, you need to ensure that the server is properly configured. Ensure that the
 appropriate authentication method is configured to allow access to the monitoring commands. You can set 
 the `monitor_key` in the `tunnelize.json`.
 
```json
{
 "tunnel": {
    "monitor_key": "secretkey"
  }
}
```

# Running commands

Once the monitoring key is set for your tunnel, you can run the following monitoring commands:

| Command                                         | Description                                                          | Example                                                                    |
| ----------------------------------------------- | -------------------------------------------------------------------- | -------------------------------------------------------------------------- |
| `tunnelize monitor system-info`                 | Retrieves system information including CPU usage, memory, and uptime | `tunnelize monitor system-info`                                            |
| `tunnelize monitor list-endpoints`              | Lists all configured endpoints on the server                         | `tunnelize monitor list-endpoints`                                         |
| `tunnelize monitor list-tunnels`                | Lists all active tunnels                                             | `tunnelize monitor list-tunnels`                                           |
| `tunnelize monitor get-tunnel tunnel_id`        | Retrieves information about a specific tunnel by ID                  | `tunnelize monitor get-tunnel 123e4567-e89b-12d3-a456-426614174000`        |
| `tunnelize monitor disconnect-tunnel tunnel_id` | Disconnects a specific tunnel by ID                                  | `tunnelize monitor disconnect-tunnel 123e4567-e89b-12d3-a456-426614174001` |
| `tunnelize monitor list-clients`                | Lists all connected clients                                          | `tunnelize monitor list-clients`                                           |
| `tunnelize monitor get-client client_id`        | Retrieves information about a specific client by ID                  | `tunnelize monitor get-client 123e4567-e89b-12d3-a456-426614174002`        |
| `tunnelize monitor list-links`                  | Lists all active links                                               | `tunnelize monitor list-links`                                             |
| `tunnelize monitor get-link link_id`            | Retrieves information about a specific link by ID                    | `tunnelize monitor get-link 123e4567-e89b-12d3-a456-426614174003`          |
| `tunnelize monitor disconnect-link link_id`     | Disconnects a specific link by ID                                    | `tunnelize monitor disconnect-link 123e4567-e89b-12d3-a456-426614174004`   |

Note that response from all of the commands is JSON meaning it can be piped for further processing.