const { connect } = Deno;

async function main() {
    const conn = await connect({ hostname: "127.0.0.1", port: 4000 });
    const encoder = new TextEncoder();
    const decoder = new TextDecoder();

    // Send "hello" to the server
    await conn.write(encoder.encode("hello"));

    // Wait for the response
    const buffer = new Uint8Array(1024);
    const bytesRead = await conn.read(buffer);
    if (bytesRead !== null) {
        const response = decoder.decode(buffer.subarray(0, bytesRead));
        console.log("Received:", response);
    }

    // Close the connection
    conn.close();
}

main();