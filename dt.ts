import { serve } from "https://deno.land/std/http/server.ts";
import { readFileStr } from "https://deno.land/std/fs/mod.ts";

const listener = Deno.listen({ port: 8000 });
console.log("HTTP server is running. Access it at: http://localhost:8000/");

for await (const conn of listener) {
  (async () => {
    const httpConn = Deno.serveHttp(conn);
    for await (const requestEvent of httpConn) {
      try {
      const url = new URL(requestEvent.request.url);
      if (url.pathname === "/image") {
        const image = await Deno.readFile("idemo.jpg");
        requestEvent.respondWith(new Response(image, { headers: { "Content-Type": "image/jpeg" } }));
      } else {
        const body = `
          <html>
            <body>
              <h1>Hello, Mata!</h1>
              <img src="/image" alt="Deno Logo" />
            </body>
          </html>
        `;
        requestEvent.respondWith(new Response(body, { headers: { "Content-Type": "text/html" } }));
      }
      } catch (error) {
        console.error(error);
      }

      
    }
  })();
}