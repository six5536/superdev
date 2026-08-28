---
type: Template
id: template-contract-mcp
title: MCP Contract Template
description: Knowledge concept skeleton — one MCP server, the tools it exposes, its errors and stability promise.
status: stable
---

---
type: McpContract
id: contract-<nnn>-mcp-<slug>
title: MCP Contract
description: <one line: which server, and what it promises>.
status: stable
---

# Server

<How the server is launched and spoken to — stdio or HTTP, one client or many — what it serves, when it exits, and what it refuses at startup rather than per call. Say here when it exposes no resources and no prompts.>

# Tools

- **`<tool_name>`** — <its arguments with defaults and limits, what comes back, and the ranking or filtering a caller can rely on.>

# Resources and prompts

<What the server exposes beyond tools. Drop this section on a server that exposes neither.>

# Errors

<How a failed call reaches the agent — an error payload against a process exit — and what the server does with input it cannot resolve.>

# Stability

<Which tool names, arguments and result shapes are promised, and how a change to any of them is signalled.>
