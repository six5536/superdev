# Overview

* [Project Overview][sokf:project-overview] - what superdev is and its current status.
* [Domain Glossary][sokf:glossary] - the terms the blueprint engine uses — blueprint, capability, provider, provenance, component, owned file, scaffold, project template, template adoption, skill pack, knowledge-carried skill, content pack, pack source, embedded snapshot, pack item, pack layer, pack format, PROJECT.md layer, custom skill, harvest, claim, orphan — plus the search terms section, locator, hybrid search and RRF.
* [Known Constraints & Non-Goals][sokf:constraints-non-goals] - what superdev deliberately does not do, and the accepted limitations of the inherited machinery.
* [Backlog & Decided Ideas][sokf:backlog] - ideas under consideration and ideas decided against, with the reasoning.
* [Ideas](ideas/index.md) - thoughts captured for considering later, one document each; not candidate work.

# Design

* [Plans](plans/index.md) - the slice lists delivering features and the plans for one-off work outside the feature workflow, filed done when they land.
* [ADRs](adrs/index.md) - architecture decision records: the interfaces that were expensive to change, with their alternatives.
* [Contracts](contracts/index.md) - the durable contracts describing the app: promised outward in public/, binding modules in internal/.
* [Issues](issues/index.md) - open tickets, grouped by feature.
* [Architecture][sokf:architecture] - the core/binary/blueprint layering, the capability-to-provider map, the knowledge-serving side, and the files superdev keeps in a managed repo.
* [Architectural Rules][sokf:architectural-rules] - planning is side-effect free, the engine is the only place that applies, and capabilities are the user-facing names.
* [Software Components][sokf:software-components] - the Rust crates, the npm launcher and platform packages, the platform matrix, and the CI/CD workflows.
* [Configuration & Environments][sokf:configuration] - the .superdev directory — the config.toml manifest, the lock file, and the gitignored cache — plus the embeddings opt-in, the custom lists, the many-provider skills shape, the guided errors, the .mcp.json and .claude/settings.json merges, and the user-level model cache.
* [Error Handling & Logging][sokf:error-handling] - exit codes, the broken-pipe rule, the validation hook's blocking exit 2, why MCP tool failures never end the process, and how a failed apply reports what it could not undo.
* [Directory Structure][sokf:directory-structure] - what lives where in the repository.
* [Technology Stack][sokf:technology-stack] - languages, runtime and dev dependencies, and the pinned toolchain set.

# Process

* [Coding Standards][sokf:coding-standards] - prose rules, Rust and TypeScript conventions, and the code-is-canonical principle.
* [Security Requirements][sokf:security-requirements] - the vulnerability policy in brief, and the security-relevant guarantees the design makes.
* [Dependency Policy][sokf:dependency-policy] - when a dependency may be added and how its version is chosen.
* [Testing Strategy][sokf:testing-strategy] - the current test layers, the key choices behind them, and the CI platforms.
* [Development Procedure][sokf:development-procedure] - setup, the contract-driven change workflow, what to run before a PR, how this repo manages its own skills, and how it serves and searches its own knowledge.
* [Development Commands][sokf:development-commands] - the npm-script command set and the pre-PR check list's shape.
* [Issue Tracker & Triage][sokf:issue-tracker] - where issues live — one SOKF concept per ticket in the issue tracker, filed by lifecycle — plus the triage label vocabulary.
* [Definition of Done][sokf:definition-of-done] - what a change must satisfy before it merges.
* [Release Procedure][sokf:release-procedure] - the changelog gate, the release command, the irreversible push, and the tag-driven pipeline.
* [Schemas](schemas/index.md) - the structural contract for every document the process produces — what sections it carries, what its frontmatter must say, and a worked example.

# Research

* [Claude Code Stop-hook Behaviour][sokf:research-001-claude-code-stop-hook-behaviour] - the Stop hook's payload, the consecutive-block cap and what resets it, the CLAUDE_CODE_STOP_HOOK_BLOCK_CAP variable, exit 2 under stop_hook_active, CLAUDE_PROJECT_DIR, and session_id across resume.

<!-- sokf:links -->
[sokf:architectural-rules]: /knowledge/architectural-rules.md
[sokf:architecture]: /knowledge/architecture.md
[sokf:backlog]: /knowledge/backlog.md
[sokf:coding-standards]: /knowledge/coding-standards.md
[sokf:configuration]: /knowledge/configuration.md
[sokf:constraints-non-goals]: /knowledge/constraints-non-goals.md
[sokf:definition-of-done]: /knowledge/definition-of-done.md
[sokf:dependency-policy]: /knowledge/dependency-policy.md
[sokf:development-commands]: /knowledge/development-commands.md
[sokf:development-procedure]: /knowledge/development-procedure.md
[sokf:directory-structure]: /knowledge/directory-structure.md
[sokf:error-handling]: /knowledge/error-handling.md
[sokf:glossary]: /knowledge/glossary.md
[sokf:issue-tracker]: /knowledge/issue-tracker.md
[sokf:project-overview]: /knowledge/project-overview.md
[sokf:release-procedure]: /knowledge/release-procedure.md
[sokf:research-001-claude-code-stop-hook-behaviour]: /knowledge/research/research-001-claude-code-stop-hook-behaviour.md
[sokf:security-requirements]: /knowledge/security-requirements.md
[sokf:software-components]: /knowledge/software-components.md
[sokf:technology-stack]: /knowledge/technology-stack.md
[sokf:testing-strategy]: /knowledge/testing-strategy.md
