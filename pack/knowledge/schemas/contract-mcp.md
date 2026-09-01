---
type: Schema
id: schema-contract-mcp
title: MCP Contract Schema
description: One MCP server — its transport, the tools it exposes, how failures are reported and the stability promise, a public contract.
---

# MCP Contract Schema

Structural rules for one public Model Context Protocol contract, filed at
`contract-{nnn}-mcp-{slug}`, a public contract placed in its lifecycle folder by `superdev validate --fix`. The caller is an agent, so the tool names and
their arguments are as much a promised surface as any command or endpoint.

<!-- sokf:include contract-style -->
**Contract style — a contract is a binding surface, not a
specification** (superdev ADR-029):

- Each normative statement MUST use an RFC 2119 modal verb, one
  requirement per sentence.
- An enumerable surface — commands, flags, keys, types, error cases,
  limits — MUST be defined in the kind's native structured form: a code
  block, table or list. Prose, doc comments included, describes and
  MUST NOT define.
- A contract MUST bind only what callers rely on; behaviour a contract
  does not list is the code's to decide.
- A contract MUST link the ADR behind each decision and MUST NOT
  restate the ADR's reasoning.
<!-- /sokf:include -->

````yaml
description: >
  One MCP server offered to agents — its transport and lifecycle, the tools it
  exposes with their arguments and results, how a failure reaches the caller,
  and what is promised stable.
line-limit: 400

frontmatter:
  type:
    required: true
    const: McpContract
  id:
    required: true
    pattern: '^contract-\d{3}-mcp-[a-z0-9-]+$'
    description: >
      contract-{nnn}-mcp-{slug}, the slug naming which MCP server. The
      number is the next free one across every contract, public and
      internal together and every lifecycle folder — a duplicate is
      an error.
  title:
    required: true
  description:
    required: true
  lifecycle:
    enum: [active, deprecated]

sections-ordered: true
sections:
  - heading-pattern: '^MCP contract: .+$'
    level: 1
    required: true
    content: prose
    description: >
      One paragraph: the surface this contract binds and for whom —
      link the ADRs behind it.
  - heading: "Server"
    level: 2
    required: true
    content: prose
    content-pattern: '\b(MUST|SHALL|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL)\b'
    description: >
      How the server is launched and spoken to — stdio or HTTP, one client or
      many — what it serves, when it exits, and what it refuses at startup
      rather than per call.
  - heading: "Tools"
    level: 2
    required: true
    content: code
    block-language: json
    block-entry-keys: [about, arguments, result]
    description: >
      The definition of the served tools, keyed by tool name. One entry per
      tool the server offers, each carrying `about`, `arguments` as a map of
      argument name to `{type, required, about}`, and `result` — what comes
      back. A caller reproduces every call from this block alone; the
      ranking, filtering and limits a caller relies on are stated in prose
      around it.
  - heading: "Resources and prompts"
    level: 2
    content: prose
    description: >
      What the server exposes beyond tools. Omit the section on a server that
      exposes neither, and say so under Server instead.
  - heading: "Errors"
    level: 2
    required: true
    content: prose
    content-pattern: '\b(MUST|SHALL|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL)\b'
    description: >
      How a failed call reaches the agent — an error payload against a process
      exit — and what the server does with input it cannot resolve.
  - heading: "Stability"
    level: 2
    required: true
    content: prose
    content-pattern: '\b(MUST|SHALL|SHOULD|MAY|REQUIRED|RECOMMENDED|OPTIONAL)\b'
    description: >
      Which tool names, arguments and result shapes are promised, and how a
      change to any of them is signalled.

example: |
  ---
  type: McpContract
  id: contract-001-mcp-widget
  title: MCP Contract
  description: The widget MCP server — two read-only tools over stdio.
  lifecycle: active
  ---

  # MCP contract: widget

  The widget MCP server: two read-only tools over stdio.

  ## Server

  `widget mcp` serves one stdio client and MUST exit 0 when that client closes
  stdin. A missing widget store MUST fail at startup rather than at every tool
  call, because a client cannot act on the latter.

  ## Tools

  ```json
  {
    "widget_search": {
      "about": "Search the widgets, best first.",
      "arguments": {
        "query": { "type": "string", "required": true,
                   "about": "What to look for." },
        "limit": { "type": "integer", "required": false,
                   "about": "Most hits to return; 8 by default, clamped to 1..50." }
      },
      "result": "One block per hit: the widget id, its name, and the score."
    },
    "widget_read": {
      "about": "Read one widget whole.",
      "arguments": {
        "id": { "type": "string", "required": true, "about": "The widget's id." }
      },
      "result": "The widget, or the near-miss candidates when no widget carries that id."
    }
  }
  ```

  Results MUST be grouped by widget, strongest first, and `limit` MUST be
  clamped rather than refused.

  ## Resources and prompts

  None. The server exposes tools only, so a client that lists resources gets an
  empty set rather than an error.

  ## Errors

  A tool failure MUST be an MCP error payload, never a process exit: the
  client stays connected and can call again. An unknown id MUST come back with
  near-miss candidates rather than an empty result.

  ## Stability

  Unreleased. Tool names, their arguments and their result shapes MAY change
  without notice until 1.0, after which a removed argument MUST get one minor
  release of deprecation first.
````
