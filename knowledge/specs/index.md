# Specs

* [CLI Core & Blueprint Engine](2026-08-11-cli-core-blueprint-engine-design.md) - design for superdev's setup/management CLI — the manifest, the component model, and init/status/sync/update.
* [AOKF MCP Server](2026-08-11-aokf-mcp-server-design.md) - design for the read-side AOKF MCP server — hybrid search, graph, the Rust validator, and the search-first AGENTS.md switchover.
* [Skill Pack](2026-08-12-skill-pack-design.md) - design for the skills capability — five skills and the validation hook shipped as owned repo files, with a PROJECT.md extension layer and a per-skill custom opt-out.
* [Blueprint Migrations](2026-08-12-blueprint-migrations-design.md) - design for evolving a managed repo — components declare what they own, the lock's leftovers are pruned or released, and CLAUDE.md imports AGENTS.md so Claude Code reads it at all.
* [Workflows Provider Default](2026-08-17-workflows-provider-default-design.md) - design for defaulting the workflows capability to mattpocock-skills as materialised repo files, with superpowers kept as a supported plugin-based secondary.
