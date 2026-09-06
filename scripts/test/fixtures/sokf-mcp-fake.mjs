import { appendFileSync, existsSync, writeFileSync } from "node:fs";
import readline from "node:readline";

const [mode = "normal", logPath = "", markerPath = ""] = process.argv.slice(2);
if (logPath) appendFileSync(logPath, `start ${process.cwd()}\n`);
let calls = 0;
const lines = readline.createInterface({ input: process.stdin });

function send(value) {
  const line = `${JSON.stringify(value)}\n`;
  if (mode === "chunked") {
    process.stdout.write(line.slice(0, 7));
    process.stdout.write(line.slice(7));
  } else {
    process.stdout.write(line);
  }
}

lines.on("line", (line) => {
  const message = JSON.parse(line);
  if (message.method === "initialize") {
    send({
      jsonrpc: "2.0",
      id: message.id,
      result: {
        protocolVersion: mode === "incompatible" ? "1900-01-01" : "2025-06-18",
        capabilities: { tools: {} },
        serverInfo: { name: "fake", version: "1" },
      },
    });
    return;
  }
  if (message.method === "notifications/initialized") return;
  if (message.method !== "tools/call") return;
  calls += 1;
  if (mode === "malformed") {
    process.stdout.write("not-json\n");
    return;
  }
  if (mode === "exit") {
    process.stderr.write("x".repeat(70 * 1024));
    process.exit(7);
  }
  if (mode === "hang-once" && markerPath && !existsSync(markerPath)) {
    writeFileSync(markerPath, "hung");
    return;
  }
  send({
    jsonrpc: "2.0",
    method: "notifications/message",
    params: { level: "info" },
  });
  send({
    jsonrpc: "2.0",
    id: message.id,
    result: { content: [{ type: "text", text: `${message.params.name}:${calls}` }] },
  });
});
