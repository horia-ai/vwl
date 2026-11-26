export * from "./stripe-service.js";
export * from "./server.js";

import { runServer } from "./server.js";

if (process.argv[1]?.endsWith("index.js") || process.argv[1]?.endsWith("index.ts")) {
  runServer().catch((err) => {
    console.error("Fatal error starting Stripe MCP server:", err);
    process.exit(1);
  });
}
