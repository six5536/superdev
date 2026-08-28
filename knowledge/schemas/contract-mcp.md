---
type: Schema
id: schema-contract-mcp
title: MCP Contract Schema
description: One MCP server — its transport, the tools it exposes, how failures are reported and the stability promise, in knowledge/contracts/public/.
---

# MCP Contract Schema

Structural rules for one public Model Context Protocol contract, filed at
`knowledge/contracts/public/contract-{nnn}-mcp-{slug}.md`. The caller is an agent, so the tool names and
their arguments are as much a promised surface as any command or endpoint.

````yaml
description: >
  One MCP server offered to agents — its transport and lifecycle, the tools it
  exposes with their arguments and results, how a failure reaches the caller,
  and what is promised stable.
line-limit: 400

frontmatter:
  type:
    const: McpContract
  id:
    pattern: '^contract-\d{3}-mcp-[a-z0-9-]+$'
    description: >
      contract-{nnn}-mcp-{slug}, the slug naming which MCP server. The
      number is the next free one across knowledge/contracts/, public and
      private together.
  status:
    enum: [draft, stable, deprecated]

sections-ordered: true
sections:
  - heading: "Server"
    level: 1
    required: true
    content: prose
    description: >
      How the server is launched and spoken to — stdio or HTTP, one client or
      many — what it serves, when it exits, and what it refuses at startup
      rather than per call.
  - heading: "Tools"
    level: 1
    required: true
    content: bullet-list
    description: >
      One entry per tool: its name, its arguments with defaults and limits,
      what comes back, and the ranking or filtering a caller can rely on.
  - heading: "Resources and prompts"
    level: 1
    content: prose
    description: >
      What the server exposes beyond tools. Omit the section on a server that
      exposes neither, and say so under Server instead.
  - heading: "Errors"
    level: 1
    required: true
    content: prose
    description: >
      How a failed call reaches the agent — an error payload against a process
      exit — and what the server does with input it cannot resolve.
  - heading: "Stability"
    level: 1
    required: true
    content: prose
    description: >
      Which tool names, arguments and result shapes are promised, and how a
      change to any of them is signalled.

example: |
  ---
  type: McpContract
  id: contract-001-mcp-widget
  title: MCP Contract
  description: The widget MCP server — two read-only tools over stdio.
  status: draft
  ---

  # Server

  `widget mcp` serves one stdio client and exits 0 when that client closes
  stdin. A missing widget store fails at startup rather than at every tool
  call, because a client cannot act on the latter.

  # Tools

  - **`widget_search`** — `query`, optional `limit` (8 by default, clamped to
    1..50). Results are grouped by widget, strongest first.
  - **`widget_read`** — `id`: the whole widget, or the near-miss candidates
    when no widget carries that id.

  # Resources and prompts

  None. The server exposes tools only, so a client that lists resources gets an
  empty set rather than an error.

  # Errors

  A tool failure is an MCP error payload, never a process exit: the client
  stays connected and can call again. An unknown id comes back with near-miss
  candidates rather than an empty result.

  # Stability

  Unreleased. Tool names, their arguments and their result shapes may change
  without notice until 1.0, after which a removed argument gets one minor
  release of deprecation first.
````
