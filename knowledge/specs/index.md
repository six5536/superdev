# Specs

* [CLI Core & Blueprint Engine][sokf:spec-001-cli-core-blueprint-engine] - design for superdev's setup/management CLI — the manifest, the component model, and init/status/sync/update.
* [AOKF MCP Server][sokf:spec-002-aokf-mcp-server] - design for the read-side AOKF MCP server — hybrid search, graph, the Rust validator, and the search-first AGENTS.md switchover.
* [Skill Pack][sokf:spec-003-skill-pack] - design for the skills capability — five skills and the validation hook shipped as owned repo files, with a PROJECT.md extension layer and a per-skill custom opt-out.
* [Blueprint Migrations][sokf:spec-004-blueprint-migrations] - design for evolving a managed repo — components declare what they own, the lock's leftovers are pruned or released, and CLAUDE.md imports AGENTS.md so Claude Code reads it at all.
* [Workflows Provider Default][sokf:spec-005-workflows-provider-default] - design for defaulting the workflows capability to mattpocock-skills as materialised repo files, with superpowers kept as a supported plugin-based secondary.
* [Workflows Skill Overrides][sokf:spec-006-workflows-skill-overrides] - design for provider-carried skill overrides — the mattpocock-skills component materialises embedded replacements, grilling first — installed only where that provider is.
* [Project Templates][sokf:spec-007-project-templates] - design for project templates — embedded write-once repo scaffolds init seeds a new repo from, token-substituted and disjoint from capability files — with rust-npm as the first, and the fuller knowledge seed it surfaced.
* [Knowledge-Carried Skills][sokf:spec-009-knowledge-carried-skills] - the aokf component ships the full converted skill set and the workflows capability is dropped; a manifest naming it gets a guided error.
* [Knowledge-Owned Skills][sokf:spec-008-knowledge-owned-skills] - design for aokf-carried lifecycle skills — aokf-maintain and the validation hook relocate to the knowledge capability, and a new aokf-bootstrap skill harvests a repo's stranded prose and interviews the owner to fill the seeded skeleton.
* [Agent Instructions Layer][sokf:spec-010-agent-instructions-layer] - AGENTS.md becomes the user's file reached by one ensured import; superdev's instructions live in an owned, fenced .agents/superdev.md aggregating per-capability instruction files, and code-index gains its missing agent wiring.
* [Skills Capability Cardinality][sokf:spec-011-skills-cardinality] - a capability declares whether it holds one provider or a set; skills becomes the first many-provider slot via [[skills]] entries, with the old single-table shape still parsing.
* [Bash Output Filter Capability][sokf:spec-012-bash-output-filter] - a new bash-output-filter capability, default provider rtk — a checksummed mise pin, an owned instruction file, and a managed PreToolUse rewrite hook that compacts command output before it reaches agent context.
* [Implements in the Core Vocabulary][sokf:spec-013-implements-rel] - promote implements/implemented-by into the SOKF core relationship vocabulary — spec bumped to 0.2, validator and graph taught the pair, and the issue-tracker convention realigned.
* [Externally Sourced Content Packs][sokf:spec-014-content-packs] - superdev's prose content becomes a versioned pack resolved from a pinned source, replacing or layering over an embedded snapshot the binary still carries, so a skill or template ships without a five-platform binary release.

<!-- sokf:links -->
[sokf:spec-001-cli-core-blueprint-engine]: /knowledge/specs/spec-001-cli-core-blueprint-engine.md
[sokf:spec-002-aokf-mcp-server]: /knowledge/specs/spec-002-aokf-mcp-server.md
[sokf:spec-003-skill-pack]: /knowledge/specs/spec-003-skill-pack.md
[sokf:spec-004-blueprint-migrations]: /knowledge/specs/spec-004-blueprint-migrations.md
[sokf:spec-005-workflows-provider-default]: /knowledge/specs/spec-005-workflows-provider-default.md
[sokf:spec-006-workflows-skill-overrides]: /knowledge/specs/spec-006-workflows-skill-overrides.md
[sokf:spec-007-project-templates]: /knowledge/specs/spec-007-project-templates.md
[sokf:spec-008-knowledge-owned-skills]: /knowledge/specs/spec-008-knowledge-owned-skills.md
[sokf:spec-009-knowledge-carried-skills]: /knowledge/specs/spec-009-knowledge-carried-skills.md
[sokf:spec-010-agent-instructions-layer]: /knowledge/specs/spec-010-agent-instructions-layer.md
[sokf:spec-011-skills-cardinality]: /knowledge/specs/spec-011-skills-cardinality.md
[sokf:spec-012-bash-output-filter]: /knowledge/specs/spec-012-bash-output-filter.md
[sokf:spec-013-implements-rel]: /knowledge/specs/spec-013-implements-rel.md
[sokf:spec-014-content-packs]: /knowledge/specs/spec-014-content-packs.md
