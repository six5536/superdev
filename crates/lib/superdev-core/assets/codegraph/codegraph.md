# Code Index

This repo has a codegraph code index. Query it before grepping or reading
files one by one:

- MCP: the `codegraph` server's `codegraph_explore` tool answers "how does
  X work", flows ("how does X reach Y") and area surveys in one shot,
  returning the relevant symbols' source plus call paths.
- CLI, for subagents and harnesses without MCP:
  `mise exec http:codegraph -- codegraph explore "<question>"`. Narrower
  commands: `query`, `node`, `callers`, `callees`, `impact`.
