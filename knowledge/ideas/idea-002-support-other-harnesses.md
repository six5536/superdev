---
type: Idea
id: idea-002-support-other-harnesses
title: Support other harnesses
description: Materialise superdev's skills, agent wiring and hooks for harnesses beyond Claude Code — Codex, Pi, OpenCode and the like.
status: draft
---

# Idea: support other harnesses

superdev today materialises everything for Claude Code alone: skills under
`.claude/skills/`, the hook entries in `.claude/settings.json`, the MCP
wiring in `.mcp.json`. Support other harnesses — Codex, Pi, OpenCode,
etc. — so a managed repo serves whichever agent the team runs.

## Open questions

- Which harness features the workflow depends on exist elsewhere —
  skills, Stop hooks, MCP — and what degrades without them?
- One canonical form translated per harness, or per-harness content in
  the pack?
- `AGENTS.md` is already the shared entry point several harnesses read;
  how much works today through that alone?
