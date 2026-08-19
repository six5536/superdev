// Tests for MCP tool definitions, device parsing, and client resolution.

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { parseDevices, buildClientMap, registerTools } from "../src/tools.ts";
import { DebugClient } from "../src/DebugClient.ts";
import type { DeviceConfig } from "../src/types.ts";

// ---- parseDevices tests ----

describe("parseDevices", () => {
  it("parses port-only format", () => {
    const devices = parseDevices("ios:8080,android:8081");
    expect(devices).toEqual([
      { name: "ios", url: "http://localhost:8080" },
      { name: "android", url: "http://localhost:8081" },
    ]);
  });

  it("parses full URL format", () => {
    const devices = parseDevices("ios:http://192.168.1.5:8080,android:http://10.0.0.1:8081");
    expect(devices).toEqual([
      { name: "ios", url: "http://192.168.1.5:8080" },
      { name: "android", url: "http://10.0.0.1:8081" },
    ]);
  });

  it("uses default when undefined", () => {
    const devices = parseDevices(undefined);
    expect(devices).toHaveLength(2);
    expect(devices[0]!.name).toBe("ios");
    expect(devices[1]!.name).toBe("android");
  });

  it("handles whitespace in entries", () => {
    const devices = parseDevices(" ios : 8080 , android : 8081 ");
    expect(devices[0]!.name).toBe("ios");
    expect(devices[1]!.name).toBe("android");
  });

  it("handles single device", () => {
    const devices = parseDevices("web:5173");
    expect(devices).toHaveLength(1);
    expect(devices[0]).toEqual({ name: "web", url: "http://localhost:5173" });
  });
});

// ---- buildClientMap tests ----

describe("buildClientMap", () => {
  it("creates DebugClient instances for each device", () => {
    const devices: DeviceConfig[] = [
      { name: "ios", url: "http://localhost:8080" },
      { name: "android", url: "http://localhost:8081" },
    ];
    const map = buildClientMap(devices);
    expect(map.size).toBe(2);
    expect(map.get("ios")).toBeInstanceOf(DebugClient);
    expect(map.get("android")).toBeInstanceOf(DebugClient);
    expect(map.get("ios")!.baseUrl).toBe("http://localhost:8080");
  });
});

// ---- registerTools integration tests (mock McpServer) ----

describe("registerTools", () => {
  const registeredTools: Map<string, { description: string; handler: Function }> = new Map();
  let mockServer: { tool: ReturnType<typeof vi.fn> };

  beforeEach(() => {
    registeredTools.clear();
    mockServer = {
      tool: vi.fn((...args: unknown[]) => {
        // McpServer.tool(name, description, schema, handler)
        const name = args[0] as string;
        const description = args[1] as string;
        const handler = args[3] as Function;
        registeredTools.set(name, { description, handler });
      }),
    };
  });

  it("registers all 8 tools", () => {
    const devices: DeviceConfig[] = [{ name: "ios", url: "http://localhost:8080" }];
    const clients = buildClientMap(devices);
    registerTools(mockServer as any, clients, devices);

    expect(registeredTools.size).toBe(8);
    expect([...registeredTools.keys()].sort()).toEqual(["debug_action", "debug_clear_logs", "debug_devices", "debug_logs", "debug_navigate", "debug_ping", "debug_screenshot", "debug_state"]);
  });

  describe("tool handlers", () => {
    const mockFetch = vi.fn();

    beforeEach(() => {
      vi.stubGlobal("fetch", mockFetch);
      mockFetch.mockReset();
      const devices: DeviceConfig[] = [{ name: "ios", url: "http://localhost:8080" }];
      const clients = buildClientMap(devices);
      registerTools(mockServer as any, clients, devices);
    });

    afterEach(() => {
      vi.restoreAllMocks();
    });

    it("debug_ping returns JSON on success", async () => {
      const ping = { status: "ok", platform: "ios", appId: "com.gt", timestamp: "t" };
      mockFetch.mockResolvedValueOnce(new Response(JSON.stringify(ping), { status: 200 }));

      const handler = registeredTools.get("debug_ping")!.handler;
      const result = await handler({ platform: "ios" });
      expect(result.content[0].type).toBe("text");
      expect(JSON.parse(result.content[0].text)).toEqual(ping);
    });

    it("debug_ping returns error when unreachable", async () => {
      mockFetch.mockRejectedValueOnce(new Error("ECONNREFUSED"));

      const handler = registeredTools.get("debug_ping")!.handler;
      const result = await handler({ platform: "ios" });
      expect(result.isError).toBe(true);
      expect(result.content[0].text).toMatch(/Not reachable/);
    });

    it("debug_ping errors on unknown platform", async () => {
      const handler = registeredTools.get("debug_ping")!.handler;
      await expect(handler({ platform: "windows" })).rejects.toThrow('Unknown platform "windows"');
    });

    it("debug_state returns JSON state", async () => {
      const state = { routes: "/home" };
      mockFetch.mockResolvedValueOnce(new Response(JSON.stringify(state), { status: 200 }));

      const handler = registeredTools.get("debug_state")!.handler;
      const result = await handler({ platform: "ios" });
      expect(JSON.parse(result.content[0].text)).toEqual(state);
    });

    it("debug_screenshot returns image content", async () => {
      const pngData = new Uint8Array([0x89, 0x50, 0x4e, 0x47]);
      mockFetch.mockResolvedValueOnce(new Response(pngData, { status: 200 }));

      const handler = registeredTools.get("debug_screenshot")!.handler;
      const result = await handler({ platform: "ios" });
      expect(result.content[0].type).toBe("image");
      expect(result.content[0].mimeType).toBe("image/png");
      expect(typeof result.content[0].data).toBe("string"); // base64
    });

    it("debug_navigate posts route", async () => {
      const nav = { navigated: true, route: "/config" };
      mockFetch.mockResolvedValueOnce(new Response(JSON.stringify(nav), { status: 200 }));

      const handler = registeredTools.get("debug_navigate")!.handler;
      const result = await handler({ platform: "ios", route: "/config" });
      expect(JSON.parse(result.content[0].text)).toEqual(nav);
    });

    it("debug_action posts action with payload", async () => {
      mockFetch.mockResolvedValueOnce(new Response(JSON.stringify({ done: true }), { status: 200 }));

      const handler = registeredTools.get("debug_action")!.handler;
      const result = await handler({ platform: "ios", type: "reset", payload: { x: 1 } });
      expect(JSON.parse(result.content[0].text)).toEqual({ done: true });
    });

    it("debug_clear_logs clears", async () => {
      mockFetch.mockResolvedValueOnce(new Response(JSON.stringify({ cleared: true }), { status: 200 }));

      const handler = registeredTools.get("debug_clear_logs")!.handler;
      const result = await handler({ platform: "ios" });
      expect(JSON.parse(result.content[0].text)).toEqual({ cleared: true });
    });

    it("debug_devices lists devices with reachability", async () => {
      // ping will be called for reachability — make it succeed
      const ping = { status: "ok", platform: "ios", appId: "com.gt", timestamp: "t" };
      mockFetch.mockResolvedValueOnce(new Response(JSON.stringify(ping), { status: 200 }));

      const handler = registeredTools.get("debug_devices")!.handler;
      const result = await handler({});
      const parsed = JSON.parse(result.content[0].text);
      expect(parsed.devices).toHaveLength(1);
      expect(parsed.devices[0].name).toBe("ios");
      expect(parsed.devices[0].connected).toBe(true);
    });

    it("debug_devices shows disconnected when unreachable", async () => {
      mockFetch.mockRejectedValueOnce(new Error("ECONNREFUSED"));

      const handler = registeredTools.get("debug_devices")!.handler;
      const result = await handler({});
      const parsed = JSON.parse(result.content[0].text);
      expect(parsed.devices[0].connected).toBe(false);
    });
  });
});
