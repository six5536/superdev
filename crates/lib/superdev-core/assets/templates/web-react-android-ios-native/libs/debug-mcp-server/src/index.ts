#!/usr/bin/env node
// MCP server entry point — stdio transport.

import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
import { parseDevices, buildClientMap, registerTools } from "./tools.ts";

const devices = parseDevices(process.env.DEBUG_DEVICES);
const clients = buildClientMap(devices);

const server = new McpServer({
  name: "native-mobile-debug",
  version: "0.1.0",
});

registerTools(server, clients, devices);

const transport = new StdioServerTransport();
await server.connect(transport);

// Log to stderr (MCP convention — stdout is protocol)
process.stderr.write(`native-debug MCP server started. Devices: ${devices.map((d) => `${d.name}→${d.url}`).join(", ")}\n`);
