# TCP endpoint

TCP endpoint is a listener for TCP traffic. When this endpoint is started, it will listen to the client connections on
a specified port range. When a client connects to a specific port, server will look for a conected tunnel on that port
and if there is such a tunnel it will create a link between them and route data.