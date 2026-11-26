import { Server } from "@modelcontextprotocol/sdk/server/index.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import {
  CallToolRequestSchema,
  ListToolsRequestSchema
} from "@modelcontextprotocol/sdk/types.js";
import {
  calculateWorkAllocation,
  createFiatPurchaseRecord,
  validateWebhookPayload
} from "./stripe-service.js";

export function createStripeMcpServer(): Server {
  const server = new Server(
    {
      name: "vwl-stripe-mcp",
      version: "0.1.0"
    },
    {
      capabilities: {
        tools: {}
      }
    }
  );

  server.setRequestHandler(ListToolsRequestSchema, async () => {
    return {
      tools: [
        {
          name: "stripe_calculate_work",
          description: "Calculate WORK units to be credited for a given fiat payment amount (Gate 9 / Fiat Onramp)",
          inputSchema: {
            type: "object",
            properties: {
              amountCents: { type: "number", description: "Fiat amount in cents (e.g. 1000 = $10.00)" },
              currency: { type: "string", description: "ISO currency code (usd or eur)" }
            },
            required: ["amountCents", "currency"]
          }
        },
        {
          name: "stripe_record_funding",
          description: "Record a verified fiat payment intent as an idempotent WORK funding receipt",
          inputSchema: {
            type: "object",
            properties: {
              purchaseId: { type: "string", description: "Unique payment intent ID (e.g. pi_...)" },
              projectId: { type: "string", description: "Target VWL Project ID" },
              amountCents: { type: "number", description: "Fiat amount in cents" },
              currency: { type: "string", description: "ISO currency code" }
            },
            required: ["purchaseId", "projectId", "amountCents", "currency"]
          }
        },
        {
          name: "stripe_validate_webhook",
          description: "Verify and extract WORK funding details from a Stripe payment_intent.succeeded webhook payload",
          inputSchema: {
            type: "object",
            properties: {
              payload: { type: "object", description: "Raw Stripe event payload" }
            },
            required: ["payload"]
          }
        }
      ]
    };
  });

  server.setRequestHandler(CallToolRequestSchema, async (request) => {
    const { name, arguments: args } = request.params;

    switch (name) {
      case "stripe_calculate_work": {
        const { amountCents, currency } = args as { amountCents: number; currency: string };
        const workUnits = calculateWorkAllocation(amountCents, currency);
        return {
          content: [
            {
              type: "text",
              text: JSON.stringify({ amountCents, currency, workUnits, ratePerCent: 10 }, null, 2)
            }
          ]
        };
      }

      case "stripe_record_funding": {
        const params = args as any;
        const record = createFiatPurchaseRecord(params);
        return {
          content: [
            {
              type: "text",
              text: JSON.stringify(record, null, 2)
            }
          ]
        };
      }

      case "stripe_validate_webhook": {
        const { payload } = args as { payload: any };
        const result = validateWebhookPayload(payload);
        return {
          content: [
            {
              type: "text",
              text: JSON.stringify(result, null, 2)
            }
          ]
        };
      }

      default:
        throw new Error(`Unknown tool: ${name}`);
    }
  });

  return server;
}

export async function runServer(): Promise<void> {
  const server = createStripeMcpServer();
  const transport = new StdioServerTransport();
  await server.connect(transport);
  console.error("VWL Stripe MCP server running on stdio");
}
