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
**Contract style — a contract defines its interface** (superdev
ADR-033, ADR-042, ADR-043, ADR-044):

- A contract's Definition MUST be one or more source includes of the
  regions that declare the interface, and MUST NOT carry an authored
  block; a caller reads the interface from the contract and reproduces
  it from the source the contract carries.
- A region MUST be bounded by `sokf:begin <name>` and `sokf:end <name>`
  in the source's own comment syntax. What is not marked is not
  promised.
- A doc comment inside an included region is contract text: a MUST
  there binds as a MUST in Behaviour does.
- Prose MUST describe and MUST NOT define. Behaviour MUST carry what no
  single element can say and what no include reaches — stability,
  consumers, behaviour across elements, exit codes, error semantics —
  each normative statement with an RFC 2119 modal verb, one requirement
  per sentence.
- Behaviour MUST cover what the schema's checklist names for the
  contract's kind, one `###` per item that applies.
- A contract MUST bind what it names and MUST NOT state how the
  interface is built inside.
- The Definition is bound by its include. The project MUST bind each
  Behaviour promise by a test of the behaviour it promises.
- A built-from source unreadable as a surface MUST be rendered by a
  generator that writes `sokf:generated-by <what>` in the rendering's
  leading lines, and the rendering MUST be proved current by a test.
- A Behaviour or Stability statement whose behaviour is unbuilt MAY
  carry `PENDING` in uppercase beside its modal verb, naming the issue
  or plan slice in parentheses, and MUST NOT once the feature settles; a
  definition element carries none.
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
