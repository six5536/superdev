// MCP tool definitions and handlers for native debug server.

import { z } from "zod";
import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { DebugClient } from "./DebugClient.ts";
import type { DeviceConfig, DeviceStatus } from "./types.ts";

/**
 * Parse DEBUG_DEVICES env var into device configs.
 * Format: "ios:8080,android:8081" or "ios:http://localhost:8080,android:http://10.0.0.1:8081"
 */
export function parseDevices(envVar?: string): DeviceConfig[] {
  const raw = envVar ?? "ios:8080,android:8081";
  return raw.split(",").map((entry) => {
    const [name, ...rest] = entry.trim().split(":");
    const urlPart = rest.join(":");
    const url = urlPart.startsWith("http") ? urlPart : `http://localhost:${urlPart}`;
    return { name: name.trim(), url };
  });
}

/**
 * Build a map of platform name → DebugClient from device configs.
 */
export function buildClientMap(devices: DeviceConfig[]): Map<string, DebugClient> {
  const map = new Map<string, DebugClient>();
  for (const d of devices) {
    map.set(d.name, new DebugClient(d.url));
  }
  return map;
}

/**
 * Resolve a DebugClient by platform name, or throw a user-friendly error.
 */
function resolveClient(clients: Map<string, DebugClient>, platform: string): DebugClient {
  const client = clients.get(platform);
  if (!client) {
    const available = [...clients.keys()].join(", ");
    throw new Error(`Unknown platform "${platform}". Available: ${available}`);
  }
  return client;
}

/**
 * Helper: wrap a JSON result as MCP text content.
 */
function jsonResult(data: unknown): { content: Array<{ type: "text"; text: string }> } {
  return {
    content: [{ type: "text" as const, text: JSON.stringify(data, null, 2) }],
  };
}

/**
 * Helper: wrap an error as MCP error result.
 */
function errorResult(message: string): {
  content: Array<{ type: "text"; text: string }>;
  isError: true;
} {
  return {
    content: [{ type: "text" as const, text: message }],
    isError: true,
  };
}

/**
 * Register all debug tools on an McpServer instance.
 */
export function registerTools(server: McpServer, clients: Map<string, DebugClient>, devices: DeviceConfig[]): void {
  // --- debug_ping ---
  server.tool("debug_ping", "Check if a native debug server is reachable", { platform: z.string().describe("Platform name (e.g. ios, android)") }, async ({ platform }) => {
    const client = resolveClient(clients, platform);
    try {
      const result = await client.ping();
      return jsonResult(result);
    } catch (err) {
      return errorResult(`Not reachable: ${err instanceof Error ? err.message : String(err)}`);
    }
  });

  // --- debug_state ---
  server.tool(
    "debug_state",
    "Get app state from a native debug server (all keys or a specific key)",
    {
      platform: z.string().describe("Platform name (e.g. ios, android)"),
      key: z.string().optional().describe("Specific state key (e.g. routes, audio). Omit for all state."),
    },
    async ({ platform, key }) => {
      const client = resolveClient(clients, platform);
      try {
        const result = await client.getState(key);
        return jsonResult(result);
      } catch (err) {
        return errorResult(`Failed to get state: ${err instanceof Error ? err.message : String(err)}`);
      }
    },
  );

  // --- debug_logs ---
  server.tool(
    "debug_logs",
    "Get buffered log entries from a native debug server",
    {
      platform: z.string().describe("Platform name (e.g. ios, android)"),
      since: z.string().optional().describe("ISO 8601 timestamp — only return entries after this time"),
    },
    async ({ platform, since }) => {
      const client = resolveClient(clients, platform);
      try {
        const result = await client.getLogs(since);
        return jsonResult(result);
      } catch (err) {
        return errorResult(`Failed to get logs: ${err instanceof Error ? err.message : String(err)}`);
      }
    },
  );

  // --- debug_screenshot ---
  server.tool("debug_screenshot", "Capture a screenshot from a native app via its debug server", { platform: z.string().describe("Platform name (e.g. ios, android)") }, async ({ platform }) => {
    const client = resolveClient(clients, platform);
    try {
      const pngBuffer = await client.getScreenshot();
      return {
        content: [
          {
            type: "image" as const,
            data: pngBuffer.toString("base64"),
            mimeType: "image/png",
          },
        ],
      };
    } catch (err) {
      return errorResult(`Screenshot failed: ${err instanceof Error ? err.message : String(err)}`);
    }
  });

  // --- debug_navigate ---
  server.tool(
    "debug_navigate",
    "Navigate a native app to a specific route via its debug server",
    {
      platform: z.string().describe("Platform name (e.g. ios, android)"),
      route: z.string().describe("Route to navigate to (e.g. /config, /hearing-assessment)"),
    },
    async ({ platform, route }) => {
      const client = resolveClient(clients, platform);
      try {
        const result = await client.navigate(route);
        return jsonResult(result);
      } catch (err) {
        return errorResult(`Navigate failed: ${err instanceof Error ? err.message : String(err)}`);
      }
    },
  );

  // --- debug_action ---
  server.tool(
    "debug_action",
    "Trigger an app-defined action via a native debug server",
    {
      platform: z.string().describe("Platform name (e.g. ios, android)"),
      type: z.string().describe("Action type (e.g. resetProfile)"),
      payload: z.record(z.string(), z.unknown()).optional().describe("Optional JSON payload for the action"),
    },
    async ({ platform, type, payload }) => {
      const client = resolveClient(clients, platform);
      try {
        const result = await client.doAction(type, payload ?? {});
        return jsonResult(result);
      } catch (err) {
        return errorResult(`Action failed: ${err instanceof Error ? err.message : String(err)}`);
      }
    },
  );

  // --- debug_clear_logs ---
  server.tool("debug_clear_logs", "Clear the log buffer on a native debug server", { platform: z.string().describe("Platform name (e.g. ios, android)") }, async ({ platform }) => {
    const client = resolveClient(clients, platform);
    try {
      const result = await client.clearLogs();
      return jsonResult(result);
    } catch (err) {
      return errorResult(`Clear logs failed: ${err instanceof Error ? err.message : String(err)}`);
    }
  });

  // --- debug_devices ---
  server.tool("debug_devices", "List configured debug devices and their connection status", {}, async () => {
    const statuses: DeviceStatus[] = await Promise.all(
      devices.map(async (d) => {
        const client = clients.get(d.name)!;
        const connected = await client.isReachable();
        return { ...d, connected };
      }),
    );
    return jsonResult({ devices: statuses });
  });
}
