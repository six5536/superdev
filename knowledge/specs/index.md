# Specs

* [CLI Core & Blueprint Engine](2026-08-11-cli-core-blueprint-engine-design.md) - design for superdev's setup/management CLI — the manifest, the component model, and init/status/sync/update.
* [AOKF MCP Server](2026-08-11-aokf-mcp-server-design.md) - design for the read-side AOKF MCP server — hybrid search, graph, the Rust validator, and the search-first AGENTS.md switchover.
* [Skill Pack](2026-08-12-skill-pack-design.md) - design for the skills capability — five skills and the validation hook shipped as owned repo files, with a PROJECT.md extension layer and a per-skill custom opt-out.
* [Blueprint Migrations](2026-08-12-blueprint-migrations-design.md) - design for evolving a managed repo — components declare what they own, the lock's leftovers are pruned or released, and CLAUDE.md imports AGENTS.md so Claude Code reads it at all.
* [Workflows Provider Default](2026-08-17-workflows-provider-default-design.md) - design for defaulting the workflows capability to mattpocock-skills as materialised repo files, with superpowers kept as a supported plugin-based secondary.
* [Workflows Skill Overrides](2026-08-18-workflows-skill-overrides-design.md) - design for provider-carried skill overrides — the mattpocock-skills component materialises embedded replacements, grilling first — installed only where that provider is.
* [Project Templates](2026-08-18-project-templates-design.md) - design for project templates — embedded write-once repo scaffolds init seeds a new repo from, token-substituted and disjoint from capability files — with rust-npm as the first, and the fuller knowledge seed it surfaced.
* [Knowledge-Owned Skills](2026-08-18-knowledge-owned-skills-design.md) - design for aokf-carried lifecycle skills — aokf-maintain and the validation hook relocate to the knowledge capability, and a new aokf-bootstrap skill harvests a repo's stranded prose and interviews the owner to fill the seeded skeleton.
