// API response types matching the native debug server contract.

/** Ping response from /debug/ping */
export interface PingResponse {
  status: "ok";
  platform: string;
  appId: string;
  timestamp: string;
}

/** Log entry from /debug/logs */
export interface LogEntry {
  ts: string;
  level: "debug" | "info" | "warn" | "error";
  tag: string;
  message: string;
}

/** Logs response from /debug/logs */
export interface LogsResponse {
  entries: LogEntry[];
  count: number;
}

/** Navigate response from POST /debug/navigate */
export interface NavigateResponse {
  navigated: boolean;
  route: string;
}

/** Clear logs response from POST /debug/logs/clear */
export interface ClearLogsResponse {
  cleared: boolean;
}

/** Error response from any endpoint */
export interface ErrorResponse {
  error: string;
}

/** Device entry for the device registry */
export interface DeviceConfig {
  name: string;
  url: string;
}

/** Device status including connection check */
export interface DeviceStatus extends DeviceConfig {
  connected: boolean;
}
