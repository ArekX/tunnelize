# Setting up a server

To setup a server, first initialize the configuration by running `tunnelize server init`.

This will create initial default configuration in `tunnelize.json` for a server.

Server will be run by just running `tunnelize` or `tunnelize server`, after which 
tunnelize server is ready to accept connections.

# Setting up a service/daemon

Tunnelize server will keep running as long as you dont stop it if you do not restart the server.
To keep it running all of the time it is best to setup a service daemon to run it in background,
below are the common ways of setting it up (assuming you are running linux).

## Setting up in systemd

1. **Create a systemd service file for Tunnelize:**

    Create a new file named `tunnelize.service` in the `/etc/systemd/system/` directory with the following content:

    ```ini
    [Unit]
    Description=Tunnelize Service
    After=network.target

    [Service]
    Type=simple
    ExecStart=/usr/local/bin/tunnelize
    Restart=on-failure
    User=nobody
    Group=nogroup
    WorkingDirectory=/path/to/your/config

    [Install]
    WantedBy=multi-user.target
    ```

    Make sure to replace `/usr/local/bin/tunnelize` with the actual path to the Tunnelize executable and `/path/to/your/config` with the directory where `tunnelize.json` is located.

2. **Reload systemd to recognize the new service:**

    ```sh
    sudo systemctl daemon-reload
    ```

3. **Enable the Tunnelize service to start on boot:**

    ```sh
    sudo systemctl enable tunnelize
    ```

4. **Start the Tunnelize service:**

    ```sh
    sudo systemctl start tunnelize
    ```

5. **Check the status of the Tunnelize service:**

    ```sh
    sudo systemctl status tunnelize
    ```

    This command will show you the current status of the Tunnelize service, including whether it is running and any recent log messages.

## Setting up in Supervisor

Supervisor is a process control system that allows you to monitor and control a number of processes on UNIX-like operating systems. Below are the steps to set up Tunnelize to run under Supervisor.

1. **Install Supervisor:**

    If Supervisor is not already installed on your system, you can install it using your package manager. For example, on Ubuntu, you can install it with:

    ```sh
    sudo apt-get update
    sudo apt-get install supervisor
    ```

2. **Create a Supervisor Configuration File for Tunnelize:**

    Create a new configuration file for Tunnelize in the Supervisor configuration directory, typically located at `/etc/supervisor/conf.d/`. Name the file `tunnelize.conf` and add the following content:

    ```ini
    [program:tunnelize]
    command=/usr/local/bin/tunnelize server
    directory=/path/to/your/config
    autostart=true
    autorestart=true
    stderr_logfile=/var/log/tunnelize.err.log
    stdout_logfile=/var/log/tunnelize.out.log
    user=nobody
    ```

    Make sure to replace `/usr/local/bin/tunnelize` with the actual path to the Tunnelize executable and `/path/to/your/config` with the directory where `tunnelize.json` is located.

3. **Update Supervisor Configuration:**

    After creating the configuration file, update Supervisor to recognize the new service:

    ```sh
    sudo supervisorctl reread
    sudo supervisorctl update
    ```

4. **Start the Tunnelize Service:**

    Start the Tunnelize service using Supervisor:

    ```sh
    sudo supervisorctl start tunnelize
    ```

5. **Check the Status of the Tunnelize Service:**

    You can check the status of the Tunnelize  withservice:

    ```sh
    sudo supervisorctl status tunnelize
    ```

    This command will show you whether the Tunnelize service is running and provide any relevant status information.

6. **Managing the Tunnelize Service:**

    You can use Supervisor to manage the Tunnelize service with the following commands:

    - **Stop the service:**

        ```sh
        sudo supervisorctl stop tunnelize
        ```

    - **Restart the service:**

        ```sh
        sudo supervisorctl restart tunnelize
        ```

    - **View the logs:**

        ```sh
        tail -f /var/log/tunnelize.out.log
        tail -f /var/log/tunnelize.err.log
        ```

# Configuring the server