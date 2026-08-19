// Tests for DebugClient HTTP client.

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { DebugClient } from "../src/DebugClient.ts";

// Mock global fetch
const mockFetch = vi.fn();
vi.stubGlobal("fetch", mockFetch);

function jsonResponse(body: unknown, status = 200): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { "Content-Type": "application/json" },
  });
}

function pngResponse(data: Uint8Array): Response {
  return new Response(data as unknown as BodyInit, {
    status: 200,
    headers: { "Content-Type": "image/png" },
  });
}

describe("DebugClient", () => {
  let client: DebugClient;

  beforeEach(() => {
    client = new DebugClient("http://localhost:8080");
    mockFetch.mockReset();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe("ping", () => {
    it("returns PingResponse on success", async () => {
      const expected = { status: "ok", platform: "ios", appId: "com.gt", timestamp: "2026-02-27T00:00:00Z" };
      mockFetch.mockResolvedValueOnce(jsonResponse(expected));

      const result = await client.ping();
      expect(result).toEqual(expected);
      expect(mockFetch).toHaveBeenCalledWith("http://localhost:8080/debug/ping", expect.objectContaining({ signal: expect.any(AbortSignal) }));
    });

    it("throws on HTTP error", async () => {
      mockFetch.mockResolvedValueOnce(jsonResponse({ error: "bad" }, 500));
      await expect(client.ping()).rejects.toThrow("bad");
    });
  });

  describe("getState", () => {
    it("gets all state when no key", async () => {
      const state = { routes: "/home", audio: { playing: true } };
      mockFetch.mockResolvedValueOnce(jsonResponse(state));

      const result = await client.getState();
      expect(result).toEqual(state);
      expect(mockFetch).toHaveBeenCalledWith("http://localhost:8080/debug/state", expect.anything());
    });

    it("gets specific key", async () => {
      const state = { playing: true };
      mockFetch.mockResolvedValueOnce(jsonResponse(state));

      const result = await client.getState("audio");
      expect(result).toEqual(state);
      expect(mockFetch).toHaveBeenCalledWith("http://localhost:8080/debug/state/audio", expect.anything());
    });

    it("encodes key with special characters", async () => {
      mockFetch.mockResolvedValueOnce(jsonResponse({}));
      await client.getState("a/b");
      expect(mockFetch).toHaveBeenCalledWith("http://localhost:8080/debug/state/a%2Fb", expect.anything());
    });
  });

  describe("getLogs", () => {
    it("gets logs without since", async () => {
      const logs = { entries: [{ ts: "t", level: "info", tag: "app", message: "hi" }], count: 1 };
      mockFetch.mockResolvedValueOnce(jsonResponse(logs));

      const result = await client.getLogs();
      expect(result).toEqual(logs);
      expect(mockFetch).toHaveBeenCalledWith("http://localhost:8080/debug/logs", expect.anything());
    });

    it("passes since parameter", async () => {
      mockFetch.mockResolvedValueOnce(jsonResponse({ entries: [], count: 0 }));
      await client.getLogs("2026-02-27T00:00:00Z");
      expect(mockFetch).toHaveBeenCalledWith("http://localhost:8080/debug/logs?since=2026-02-27T00%3A00%3A00Z", expect.anything());
    });
  });

  describe("getScreenshot", () => {
    it("returns Buffer with PNG data", async () => {
      const pngData = new Uint8Array([0x89, 0x50, 0x4e, 0x47]);
      mockFetch.mockResolvedValueOnce(pngResponse(pngData));

      const result = await client.getScreenshot();
      expect(Buffer.isBuffer(result)).toBe(true);
      expect(result[0]).toBe(0x89);
      expect(result[1]).toBe(0x50);
    });

    it("throws on error", async () => {
      mockFetch.mockResolvedValueOnce(new Response("not found", { status: 404 }));
      await expect(client.getScreenshot()).rejects.toThrow("Screenshot failed (404)");
    });
  });

  describe("navigate", () => {
    it("posts route and returns result", async () => {
      const expected = { navigated: true, route: "/config" };
      mockFetch.mockResolvedValueOnce(jsonResponse(expected));

      const result = await client.navigate("/config");
      expect(result).toEqual(expected);
      expect(mockFetch).toHaveBeenCalledWith(
        "http://localhost:8080/debug/navigate",
        expect.objectContaining({
          method: "POST",
          body: JSON.stringify({ route: "/config" }),
        }),
      );
    });
  });

  describe("doAction", () => {
    it("posts action with payload", async () => {
      const expected = { success: true };
      mockFetch.mockResolvedValueOnce(jsonResponse(expected));

      const result = await client.doAction("resetProfile", { userId: "123" });
      expect(result).toEqual(expected);
      expect(mockFetch).toHaveBeenCalledWith(
        "http://localhost:8080/debug/action",
        expect.objectContaining({
          method: "POST",
          body: JSON.stringify({ type: "resetProfile", payload: { userId: "123" } }),
        }),
      );
    });

    it("sends empty payload by default", async () => {
      mockFetch.mockResolvedValueOnce(jsonResponse({ done: true }));
      await client.doAction("resetProfile");
      expect(mockFetch).toHaveBeenCalledWith(
        "http://localhost:8080/debug/action",
        expect.objectContaining({
          body: JSON.stringify({ type: "resetProfile", payload: {} }),
        }),
      );
    });
  });

  describe("clearLogs", () => {
    it("clears logs", async () => {
      mockFetch.mockResolvedValueOnce(jsonResponse({ cleared: true }));
      const result = await client.clearLogs();
      expect(result).toEqual({ cleared: true });
    });
  });

  describe("isReachable", () => {
    it("returns true when ping succeeds", async () => {
      mockFetch.mockResolvedValueOnce(jsonResponse({ status: "ok", platform: "ios", appId: "x", timestamp: "t" }));
      expect(await client.isReachable()).toBe(true);
    });

    it("returns false when ping fails", async () => {
      mockFetch.mockRejectedValueOnce(new Error("ECONNREFUSED"));
      expect(await client.isReachable()).toBe(false);
    });
  });

  describe("timeout handling", () => {
    it("aborts after timeout", async () => {
      const slowClient = new DebugClient("http://localhost:8080", 50);
      mockFetch.mockImplementationOnce((_url: string, init: RequestInit) => {
        return new Promise((_resolve, reject) => {
          init.signal?.addEventListener("abort", () => reject(new DOMException("aborted", "AbortError")));
        });
      });
      await expect(slowClient.ping()).rejects.toThrow();
    });
  });
});
