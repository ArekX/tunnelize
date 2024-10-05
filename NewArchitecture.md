```
tunnel - hub
    - allow tls connections
    - proxies:
        - creates http proxy
        - creates tcp proxy
        - create udp proxy
    - hub server connection
      - authenticate via auth key
      - get available services
      - request session from server for a specific tunnel service mentioning all
        things to be proxied
            - server returns unique session key or fails if tunnel or service is not supported -> exit
      - connection is ready
    - on data received from hub server:
        - receive unique one-time session key for tunnel to connect
        - hub based on the service name and the desired tunnel lets tunnel know to connect passing the session key
        - once key is accepted data transfer can begin until one side or all closes



server - hub
    - allow tls connections
    - security
        - max wait time for input for tunnel and client
        - save IPs for each login attempt and make them wait 30 mins after 5 attempts
        - set max tunnel connections

    - endpoints:
        - http
            - determine tunnel id based on the hostname
            - if authorization is needed request that as well
            - if no available tunnel for hostname or anything close connection and return http error
        - tcp
            - determine tunnel id based on connected port
            - if not available tunnel output on server and close connection
        - udp
            - determine tunnel id based on port
            - if not available tunnel output on server and do nothing (no connections on udp)
        - monitoring-api
            - axum monitoring api
            - sends monitoring requests to hub server
    - hub server:
        - when tunnel is connected, request auth key if needed
        - after successful auth, generate unique session key
        - tunnel will send all services it has and hub will reply if everything can be supported
        - tunnel is now considered connected and waiting for requests.
    - hub client:
        - each endpoint will handle its own client connections
        - when client connects to the endpoint, service will send a request to hub via channel with relevant data mentioning the tunnel id. Link request must include tunnel ID and a stream enum (TCP/UDP).
        - hub will determine which tunnel by ID, assign client ID and send a link request to the tunnel with one-time link session key
        - tunnel will make a new connection with link accept passing the session key
        - data transfer between tunnel and client will start (a link session) and it will return a termination tx which when sent will terminate the connection (use tokio::select!)
    - hub monitoring 
        - requires monitoring auth key if set
        - apis
            - system-info - returns memory, cpu consumption and other system information
            - list services - returns list of all services currently running with their names (this is accessible by monitoring auth key and tunnel auth key)
            - get service data - returns a specific service data with metrics like amount of tunnels and amount of clients
            - list service tunnels - returns a list of currently connected tunnel IDs with forwarded ports and assigned hostnames.
            - list service clients - returns a list of currently connected clients with client IDs and to which tunnel ID they are connected.
            - disconnect tunnel - forces a tunnel disconnect by ID
            - disconnect client - forces a client disconnect by ID


cli
    - add tunnelize tunnel --init-from=servername:port [-a,--auth-key=key], cli connects to the server, uses list services to get service data to initialize tunnel
    - add tunnelize tunnel -s,--service=name -p,--port=port. This will use tunnelize.json for initial config (like auth and data), and run one time tunnel instead of running it from tunnelize.json\
    - add tunnelize monitor sys-info
    - add tunnelize monitor list-services
    - add tunnelize monitor get-service name
    - add tunnelize monitor list-tunnels service-name
    - add tunnelize monitor list-clients service-name
    - add tunnelize monitor disconnect-tunnel tunnel-id
    - add tunnelize monitor disconnect-client client-id
```