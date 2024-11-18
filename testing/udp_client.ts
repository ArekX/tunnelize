const client = Deno.listenDatagram({
  port: 0,
  transport: "udp",
  hostname: "0.0.0.0",
});

const serverAddr = {
  hostname: "arekxv.name",
  port: 4051,
  transport: "udp",
};

await client.send(new TextEncoder().encode("Hello from client"), serverAddr);

console.log("waiting for response...");
const [response] = await client.receive();
console.log(`Received: ${new TextDecoder().decode(response)}`);
client.close();