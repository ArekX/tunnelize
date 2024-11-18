// Simple TCP echo server using Deno
const listener = Deno.listen({ port: 8081 });
console.log("TCP server running on localhost:8081");

// Handle a single connection then exit
const conn = await listener.accept();

// Create a buffer for reading data
const buffer = new Uint8Array(1024);
const n = await conn.read(buffer);

if (n) {
  // Convert received bytes to string and log
  const message = new TextDecoder().decode(buffer.subarray(0, n));
  console.log("Received:", message);
  
  // Echo the message back
  await conn.write(buffer.subarray(0, n));
}

// Close connection and listener
conn.close();
listener.close();
console.log("Server closed");