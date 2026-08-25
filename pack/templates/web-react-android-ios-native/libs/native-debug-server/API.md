# Native Debug Server — API Contract

Version: 1.0.0

All endpoints return JSON unless otherwise noted. Server listens on `localhost:8080` (configurable).

## Core Endpoints (provided by library)

### GET /debug/ping

Health check.

**Response 200:**
```json
{
  "status": "ok",
  "platform": "ios" | "android",
  "appId": "com.{{superdev:project-compact}}.app",
  "timestamp": "2026-02-27T12:00:00Z"
}
```

### GET /debug/logs

Return buffered log entries.

**Query parameters:**
- `since` (optional, ISO 8601) — return only entries after this timestamp

**Response 200:**
```json
{
  "entries": [
    {
      "ts": "2026-02-27T12:00:00.123Z",
      "level": "debug" | "info" | "warn" | "error",
      "tag": "Audio",
      "message": "Engine started"
    }
  ],
  "count": 1
}
```

### POST /debug/logs/clear

Clear the log buffer.

**Response 200:**
```json
{ "cleared": true }
```

### GET /debug/screenshot

Capture a screenshot of the current screen.

**Response 200:** `Content-Type: image/png` — raw PNG bytes

**Response 500:**
```json
{ "error": "Screenshot capture not available" }
```

## App-Registered Endpoints

### GET /debug/state

Return all registered state providers merged into one object.

**Response 200:**
```json
{
  "routes": { "current": "/settings", "stack": [...] },
  "audio": { "playing": false }
}
```

### GET /debug/state/{key}

Return a specific state subtree.

**Response 200:** The state object for the given key.

**Response 404:**
```json
{ "error": "Unknown state key: <key>" }
```

### GET /debug/routes

Alias for `GET /debug/state/routes`. Returns navigation state.

**Response 200:**
```json
{
  "current": "/settings",
  "stack": ["/home", "/settings"],
  "available": ["/home", "/settings", "/about", ...]
}
```

### POST /debug/navigate

Trigger navigation to a route.

**Request body:**
```json
{ "route": "/about" }
```

**Response 200:**
```json
{ "navigated": true, "route": "/about" }
```

**Response 400:**
```json
{ "error": "Missing 'route' in request body" }
```

### POST /debug/action

Trigger an app-defined action.

**Request body:**
```json
{ "type": "resetProfile", "payload": {} }
```

**Response 200:** Action-specific response JSON.

**Response 400:**
```json
{ "error": "Missing 'type' in request body" }
```

**Response 404:**
```json
{ "error": "Unknown action: <type>" }
```

## Error Responses

All errors use the format:
```json
{ "error": "<message>" }
```

| Status | Meaning |
|--------|---------|
| 400 | Bad request (missing/invalid parameters) |
| 404 | Unknown endpoint or key |
| 500 | Internal server error |

## Log Entry Schema

| Field | Type | Description |
|-------|------|-------------|
| `ts` | string (ISO 8601) | Timestamp with milliseconds |
| `level` | `"debug"` \| `"info"` \| `"warn"` \| `"error"` | Severity |
| `tag` | string | Category/component tag |
| `message` | string | Log message |

## Integration Pattern

1. Library provides: HTTP server, log buffer, screenshot capture, router
2. App registers `StateProvider` callbacks keyed by name
3. App registers `ActionHandler` callbacks keyed by type
4. Library calls providers/handlers on matching requests
5. Circular log buffer (default 1000 entries) managed by library
