
# Setting up a service

Tunnelize server will keep running as long as you dont stop it if you do not restart the server.
To keep it running all of the time it is best to setup a service daemon to run it in background,
below are the common ways of setting it up (assuming you are running linux).

## Using Systemd

Systemd is a system and service manager for Linux operating systems. It is responsible for initializing the system, managing system processes, and handling system services. Systemd provides a standardized way to manage services, including starting, stopping, enabling, and disabling them. It uses unit files to define services and their configurations, allowing for consistent and efficient service management. Systemd also handles dependencies between services, ensuring that services start in the correct order and that required services are available when needed.

Create a new file named `tunnelize.service` in the `/etc/systemd/system/` directory (check your linux distribution for correct paths) with the following content:

```ini
[Unit]
Description=Tunnelize Service
After=network.target

[Service]
Type=simple
ExecStart=/path/to/tunnelize
WorkingDirectory=/path/to/your/config
Restart=on-failure
User=nobody
Group=nogroup

[Install]
WantedBy=multi-user.target
```

Make sure to replace `/path/to/tunnelize` with the actual path to the Tunnelize executable and `/path/to/your/config` with the directory where `tunnelize.json` is located if you are running
`tunnelize server --config=path`. Set the user to the desired user which will run the process (check information about Systemd).

After setting up everything reload systemd daemon to apply the changes:

```sh
sudo systemctl daemon-reload
```

After this your service is discoverable but it is not enabled. To enable it run:

```sh
sudo systemctl enable tunnelize
```

Then start your service:

```sh
sudo systemctl start tunnelize
```

To see the logs and status of tunnelize service run:

```sh
sudo systemctl status tunnelize
```

## Using Supervisor

Supervisor is a process control system for UNIX-like operating systems. It allows you to monitor and control multiple processes, ensuring they stay running and automatically restarting them if they fail. This is particularly useful for managing long-running services and applications, providing a simple way to keep them operational without manual intervention.

Supervisor does not usually come with your linux distribution. First it must be installed. You can do so by running:

```sh
sudo apt-get update
sudo apt-get install supervisor
```

Check your linux distribution for proper installation of supervisor if not using `apt-get` (linux distributions like Ubuntu or Debian).

**Create a Supervisor Configuration File for Tunnelize:**

Create a new configuration file for Tunnelize in the Supervisor configuration directory, typically located at `/etc/supervisor/conf.d/`. Name the file `tunnelize.conf` and add the following content:

```ini
[program:tunnelize]
command=/usr/local/bin/tunnelize
directory=/path/to/your/config
autostart=true
autorestart=true
stderr_logfile=/var/log/tunnelize.err.log
stdout_logfile=/var/log/tunnelize.out.log
user=nobody
```

Make sure to replace `/usr/local/bin/tunnelize` with the actual path to the Tunnelize executable and `/path/to/your/config` with the directory where `tunnelize.json` is located.
Set the user to the desired user which will run the process.

After creating the configuration file, update Supervisor to recognize the new service:

```sh
sudo supervisorctl reread
sudo supervisorctl update
```

Start the Tunnelize service using Supervisor:

```sh
sudo supervisorctl start tunnelize
```

You can check the status of the Tunnelize  withservice:

```sh
sudo supervisorctl status tunnelize
```