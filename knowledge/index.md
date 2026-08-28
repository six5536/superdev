# Overview

* [Project Overview](project-overview.md) - what superdev is and its current status.
* [Domain Glossary](glossary.md) - the terms the blueprint engine uses — blueprint, capability, provider, provenance, component, owned file, scaffold, project template, template adoption, skill pack, knowledge-carried skill, content pack, pack source, embedded snapshot, pack item, pack layer, pack format, PROJECT.md layer, custom skill, harvest, claim, orphan — plus the search terms section, locator, hybrid search and RRF.
* [Known Constraints & Non-Goals](constraints-non-goals.md) - what superdev deliberately does not do, and the accepted limitations of the inherited machinery.
* [Backlog & Decided Ideas](backlog.md) - ideas under consideration and ideas decided against, with the reasoning.

# Design

* [Specs](specs/index.md) - design specs: permanent decision records, one per significant change.
* [Plans](plans/index.md) - the slice lists delivering specs and the plans for one-off work outside the feature workflow, tagged done when they land.
* [Decisions](decisions/index.md) - architecture decision records: the interfaces that were expensive to change, with their alternatives.
* [Contracts](contracts/index.md) - the contracts promised outward, and the per-feature interface contracts build codes against.
* [Issues](issues/index.md) - open tickets, grouped by feature.
* [Architecture](architecture.md) - the core/binary/blueprint layering, the capability-to-provider map, the knowledge-serving side, and the files superdev keeps in a managed repo.
* [Architectural Rules](architectural-rules.md) - planning is side-effect free, the engine is the only place that applies, and capabilities are the user-facing names.
* [Software Components](software-components.md) - the Rust crates, the npm launcher and platform packages, the platform matrix, and the CI/CD workflows.
* [Configuration & Environments](configuration.md) - the .superdev directory — the config.toml manifest, the lock file, and the gitignored cache — plus the embeddings opt-in, the custom lists, the many-provider skills shape, the guided errors, the .mcp.json and .claude/settings.json merges, and the user-level model cache.
* [Error Handling & Logging](error-handling.md) - exit codes, the broken-pipe rule, the validation hook's blocking exit 2, why MCP tool failures never end the process, and how a failed apply reports what it could not undo.
* [Directory Structure](directory-structure.md) - what lives where in the repository.
* [Technology Stack](technology-stack.md) - languages, runtime and dev dependencies, and the pinned toolchain set.

# Process

* [Coding Standards](coding-standards.md) - prose rules, Rust and TypeScript conventions, and the code-is-canonical principle.
* [Security Requirements](security-requirements.md) - the vulnerability policy in brief, and the security-relevant guarantees the design makes.
* [Dependency Policy](dependency-policy.md) - when a dependency may be added and how its version is chosen.
* [Testing Strategy](testing-strategy.md) - the current test layers, the key choices behind them, and the CI platforms.
* [Development Procedure](development-procedure.md) - setup, the spec-and-plan change workflow, what to run before a PR, how this repo manages its own skills, and how it serves and searches its own knowledge.
* [Development Commands](development-commands.md) - the npm-script command set and the pre-PR check list's shape.
* [Issue Tracker & Triage](issue-tracker.md) - where issues live — one SOKF concept per ticket under knowledge/issues/ — plus the triage label vocabulary.
* [Definition of Done](definition-of-done.md) - what a change must satisfy before it merges.
* [Release Procedure](release-procedure.md) - the changelog gate, the release command, the irreversible push, and the tag-driven pipeline.
* [Schemas](schemas/index.md) - the structural contract for every document the process produces — what sections it carries, what its frontmatter must say, and a worked example.
