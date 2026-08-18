# Overview

* [Project Overview](project-overview.md) - what superdev is and its current status.
* [Domain Glossary](glossary.md) - the terms the blueprint engine uses — blueprint, capability, provider, provenance, component, skill override, owned file, scaffold, project template, skill pack, materialised skill, PROJECT.md layer, custom skill, harvest, claim, orphan — plus the search terms section, locator, hybrid search and RRF.
* [Known Constraints & Non-Goals](constraints-non-goals.md) - accepted limitations of the inherited machinery; product non-goals are TBD.
* [Backlog & Decided Ideas](backlog.md) - ideas under consideration and ideas decided against, with the reasoning.

# Design

* [Specs](specs/index.md) - design specs: permanent decision records, one per significant change.
* [Architecture](architecture.md) - the core/binary/blueprint layering, the capability-to-provider map, the knowledge-serving side, and the files superdev keeps in a managed repo.
* [Architectural Rules](architectural-rules.md) - planning is side-effect free, the engine is the only place that applies, and capabilities are the user-facing names.
* [Software Components](software-components.md) - the Rust crates, the npm launcher and platform packages, the platform matrix, and the CI/CD workflows.
* [Configuration & Environments](configuration.md) - the .superdev directory — the config.toml manifest, the lock file with its owners table, and the gitignored cache — plus the workflows provider choice, the embeddings opt-in, the custom lists, the .mcp.json and .claude/settings.json merges, and the user-level model cache.
* [API Contracts](api-contracts.md) - the CLI surface — the manage verbs, the knowledge verbs, the four MCP tools, and the stability promises.
* [Error Handling & Logging](error-handling.md) - exit codes, the broken-pipe rule, the validation hook's blocking exit 2, why MCP tool failures never end the process, and how a failed apply reports what it could not undo.
* [Directory Structure](directory-structure.md) - what lives where in the repository.
* [Technology Stack](technology-stack.md) - languages, runtime and dev dependencies, and the pinned toolchain set.

# Process

* [Coding Standards](coding-standards.md) - prose rules, Rust and TypeScript conventions, and the code-is-canonical principle.
* [Security Requirements](security-requirements.md) - the vulnerability policy in brief; the security surface is TBD with the design.
* [Dependency Policy](dependency-policy.md) - when a dependency may be added and how its version is chosen.
* [Testing Strategy](testing-strategy.md) - the current test layers, the key choices behind them, and the CI platforms.
* [Development Procedure](development-procedure.md) - setup, the spec-and-plan change workflow, what to run before a PR, how this repo manages its own skills and workflows, and how it serves and searches its own knowledgebase.
* [Development Commands](development-commands.md) - the npm-script command set and the pre-PR check list's shape.
* [Issue Tracker & Triage](issue-tracker.md) - where issues live — one AOKF concept per ticket under knowledge/issues/ — plus the triage label vocabulary.
* [Domain Docs](domain-docs.md) - which bundle concepts serve as the domain docs and how engineering skills use them.
* [Definition of Done](definition-of-done.md) - what a change must satisfy before it merges.
* [Release Procedure](release-procedure.md) - the changelog gate, the release command, the irreversible push, and the tag-driven pipeline.
