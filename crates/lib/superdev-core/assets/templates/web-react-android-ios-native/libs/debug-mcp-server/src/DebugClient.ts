// Typed HTTP client for the native debug server API.

import type { PingResponse, LogsResponse, NavigateResponse, ClearLogsResponse, ErrorResponse } from "./types.ts";

const DEFAULT_TIMEOUT = 2000;

/**
 * HTTP client for a single native debug server.
 * Each instance targets one base URL (one device).
 */
export class DebugClient {
  constructor(
    public readonly baseUrl: string,
    private readonly timeout = DEFAULT_TIMEOUT,
  ) {}

  /** GET /debug/ping */
  async ping(): Promise<PingResponse> {
    return this.getJSON<PingResponse>("/debug/ping");
  }

  /** GET /debug/state or /debug/state/<key> */
  async getState(key?: string): Promise<Record<string, unknown>> {
    const path = key ? `/debug/state/${encodeURIComponent(key)}` : "/debug/state";
    return this.getJSON<Record<string, unknown>>(path);
  }

  /** GET /debug/logs, optionally filtered by since timestamp */
  async getLogs(since?: string): Promise<LogsResponse> {
    const query = since ? `?since=${encodeURIComponent(since)}` : "";
    return this.getJSON<LogsResponse>(`/debug/logs${query}`);
  }

  /** GET /debug/screenshot — returns raw PNG bytes */
  async getScreenshot(): Promise<Buffer> {
    const response = await this.fetch("/debug/screenshot");
    if (!response.ok) {
      const body = await response.text();
      throw new Error(`Screenshot failed (${response.status}): ${body}`);
    }
    return Buffer.from(await response.arrayBuffer());
  }

  /** POST /debug/navigate */
  async navigate(route: string): Promise<NavigateResponse> {
    return this.postJSON<NavigateResponse>("/debug/navigate", { route });
  }

  /** POST /debug/action */
  async doAction(type: string, payload: Record<string, unknown> = {}): Promise<Record<string, unknown>> {
    return this.postJSON<Record<string, unknown>>("/debug/action", {
      type,
      payload,
    });
  }

  /** POST /debug/logs/clear */
  async clearLogs(): Promise<ClearLogsResponse> {
    return this.postJSON<ClearLogsResponse>("/debug/logs/clear", {});
  }

  /** Check if the server is reachable (returns true/false, never throws) */
  async isReachable(): Promise<boolean> {
    try {
      await this.ping();
      return true;
    } catch {
      return false;
    }
  }

  // --- Internal helpers ---

  private async fetch(path: string, init?: RequestInit): Promise<Response> {
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), this.timeout);
    try {
      return await fetch(`${this.baseUrl}${path}`, {
        ...init,
        signal: controller.signal,
      });
    } finally {
      clearTimeout(timer);
    }
  }

  private async getJSON<T>(path: string): Promise<T> {
    const response = await this.fetch(path);
    if (!response.ok) {
      const body = (await response.json()) as ErrorResponse;
      throw new Error(body.error ?? `HTTP ${response.status}`);
    }
    return (await response.json()) as T;
  }

  private async postJSON<T>(path: string, body: Record<string, unknown>): Promise<T> {
    const response = await this.fetch(path, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    });
    if (!response.ok) {
      const resp = (await response.json()) as ErrorResponse;
      throw new Error(resp.error ?? `HTTP ${response.status}`);
    }
    return (await response.json()) as T;
  }
}
