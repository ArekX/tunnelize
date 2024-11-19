const client = Deno.listenDatagram({
  port: 0,
  transport: "udp",
  hostname: "0.0.0.0",
});

const serverAddr = {
  hostname: "127.0.0.1",
  port: 5000,
  transport: "udp",
};

while(true) {
await client.send(new TextEncoder().encode("Hello from client"), serverAddr);

console.log("waiting for response...");
const [response] = await client.receive();
console.log(`Received: ${new TextDecoder().decode(response)}`);
}





client.close();