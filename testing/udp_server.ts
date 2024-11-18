const server = Deno.listenDatagram({
  port: 8081,
  transport: "udp",
  hostname: "127.0.0.1",
});

console.log("UDP server listening on 127.0.0.1:8081");

const [msg, remoteAddr] = await server.receive();
console.log(`Received: ${new TextDecoder().decode(msg)} from ${remoteAddr.hostname}:${remoteAddr.port}`);

await server.send(new TextEncoder().encode("Hello from server"), remoteAddr);
server.close();