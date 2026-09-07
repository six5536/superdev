---
type: Documentation
id: documentation
title: Documentation map
description: Superdev's user, contributor, generated-reference, and canonical-knowledge documentation surfaces with objective triggers and checks.
---

# Documentation map

## Policy

SCOPE evaluates every surface trigger for each change and records either the
owned update or a checked non-applicability rationale. BUILD edits only the
source of truth, runs generation where declared, and records the verification
command. Generated output must be clean after a second generation pass. Site
publication and package release remain outside the local workflow.

## Surfaces

### Surface: readme

- Audience and purpose: prospective and current users installing and operating Superdev.
- Kind: handwritten.
- Authored sources: `/README.md`.
- Generated outputs: none.
- Source of truth: `/README.md`.
- Triggers: public CLI, configuration, installation, workflow, migration, and product-name changes.
- Generation command: none.
- Verification command: `npm run check:docs` in `development-commands`.
- Generated output: not applicable.
- Publication owner: maintainers through the normal release process.

### Surface: contributor-guide

- Audience and purpose: contributors building, testing, and changing Superdev.
- Kind: handwritten.
- Authored sources: `/CONTRIBUTING.md`, `/knowledge/development-commands.md`.
- Generated outputs: none.
- Source of truth: `/CONTRIBUTING.md` for the command list and `/knowledge/development-commands.md` for command semantics.
- Triggers: development command, test, validation, extension, workflow, or repository-layout changes.
- Generation command: none.
- Verification command: `npm run check:docs` in `development-commands`.
- Generated output: not applicable.
- Publication owner: maintainers; repository rendering is the publication.

### Surface: canonical-knowledge

- Audience and purpose: maintainers and coding agents retrieving current architecture, operations, security, and delivery rules.
- Kind: handwritten.
- Authored sources: `/knowledge/**/*.md` excluding materialized include and link blocks.
- Generated outputs: `/knowledge/**` materialized include and link blocks.
- Source of truth: canonical concept prose plus source regions named by include blocks.
- Triggers: architecture, component, security, contract, ADR, schema, workflow, testing, command, migration, or definition-of-done changes.
- Generation command: `cargo run -- validate --fix` in `development-commands`.
- Verification command: `npm run check:validate` in `development-commands`.
- Generated output: committed.
- Publication owner: maintainers; SOKF is consumed locally through CLI, MCP, and Pi.

### Surface: cli-reference

- Audience and purpose: command-line users reading help, completions, man output, and CLI contracts.
- Kind: generated.
- Authored sources: `/crates/app/superdev/src/main.rs`, `/crates/app/superdev/src/*_cli.rs`, and source-owned Clap regions.
- Generated outputs: materialized Definition blocks in `/knowledge/contracts/public/active/contract-002-cli-superdev.md`; release help, man, and completions are build-only.
- Source of truth: Clap declarations and their doc comments.
- Triggers: command, subcommand, argument, help, exit-code, stream, or workflow protocol changes.
- Generation command: `cargo run -- validate --fix` in `development-commands`.
- Verification command: `cargo test -p superdev --test cli` and `npm run check:validate` in `development-commands`.
- Generated output: contract definitions are committed; help, man, and completions are build-only.
- Publication owner: release automation for packaged artifacts.

### Surface: changelog

- Audience and purpose: users assessing behavior and migration changes between releases.
- Kind: handwritten.
- Authored sources: `/CHANGELOG.md`.
- Generated outputs: none.
- Source of truth: `/CHANGELOG.md`.
- Triggers: user-observable feature, fix, removal, compatibility, or migration change.
- Generation command: none.
- Verification command: `npm run check:docs` in `development-commands`.
- Generated output: not applicable.
- Publication owner: maintainers through the normal release process.

### Surface: documentation-site

- Audience and purpose: users looking for a separately rendered documentation website.
- Kind: site.
- Authored sources: none; this repository currently has no documentation site.
- Generated outputs: none.
- Source of truth: none until a reviewed map change introduces a site.
- Triggers: introduction of a documentation-site source or renderer.
- Generation command: none.
- Verification command: `npm run check:docs` confirms the declared absence.
- Generated output: build-only when introduced.
- Publication owner: unassigned; deployment is outside this workflow.
