---
type: Overview
id: project-overview
title: Project Overview
description: What superdev is and its current status.
status: stable
---

superdev sets a repository up for agent-driven development and keeps that
setup current. Run inside a target repo, `init` installs the tooling — a
knowledgebase, a code index, Claude Code plugins — and records what it did;
`status`, `sync` and `update` keep the repo matching the blueprint compiled
into the binary. `mcp aokf` then serves that knowledgebase back to agents, so
they search it instead of swallowing it whole. It is opinionated for this
project's stack (Claude Code, mise, AOKF); generalisation is not a goal yet.
See
[architecture](architecture.md) for the design and
[software-components](software-components.md) for what implements it.

# Status

Unreleased. The CLI core, the blueprint engine and the AOKF MCP server's read
side are built; structured updates through MCP, the skill pack and blueprint
migrations are not. Nothing is published to npm or crates.io yet.
