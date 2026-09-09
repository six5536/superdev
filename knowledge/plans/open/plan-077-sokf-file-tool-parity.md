---
type: Plan
id: plan-077-sokf-file-tool-parity
title: Match SOKF-routed file tools to Pi's built-in contracts
description: Preserve Pi's read, edit, and write contracts across SOKF routing while retaining agent-safe knowledge mutation and bounded diagnostics.
lifecycle: open
phase: scope
branch: work/077-sokf-file-tool-parity
links:
  - rel: implements
    to: issue-077-sokf-file-tool-parity
---

# Plan: Match SOKF-routed file tools to Pi's built-in contracts

Primary issue: [issue-077-sokf-file-tool-parity][sokf:issue-077-sokf-file-tool-parity]

## Goal and boundaries

A physical knowledge path and its equivalent `sokf:<id>` address must expose Pi's built-in file-tool contract: exact source, argument preparation, pagination, truncation, matching, errors, cancellation, mutation queueing, concise results, details, metadata, and rendering. SOKF remains responsible for identity protection, generated ownership, repair, refiling, and validation, but its mutation envelopes and whole-file reports do not enter normal file-tool content.

The work includes the Pi adapter, the SOKF service and mutation boundary needed for exact-source routing, paired parity tests, bounded diagnostics, prompt metadata, shipped copies, and affected canonical knowledge. It excludes a new semantic retrieval dialect for `read`, approximate forks of Pi algorithms where factories or helpers can be reused, whole-file rewrites as recovery from failed targeted edits, weakening SOKF safety, and model-driven integration tests where direct in-process assertions suffice.

## Requirements

Full `sokf:<id>` reads return the exact UTF-8 concept source and remain mutation-ready. A section-qualified `sokf:<id>#<heading>` read returns the exact contiguous source lines belonging to that section, with `offset` and `limit` applied within the source slice. Physical and virtual aliases use one resolved physical mutation-queue key.

SOKF-routed edits retain Pi's schema and legacy argument preparation, exact-then-normalized matching, original-snapshot uniqueness and overlap checks, BOM and line-ending preservation, error and abort behavior, concise success content, and compact built-in detail shape. SOKF-routed writes retain Pi's parameters, recursive parent creation, errors, cancellation, concise content, undefined details, and renderer. Policy rejections are concise actionable errors and never recommend a whole-file fallback.

Repair, refiling, validation, and mutation diagnostics remain available for extension state and bounded operator feedback without replacing or duplicating built-in tool results. File tools inherit Pi's prompt metadata; only `sokf_search` and `sokf_graph` describe discovery, and each additional routing guideline names its governing tool exactly once.

## Contract changes

- `contract-003-api-sokf`: revise `P_concept-address` so concept identities resolve to exact source and section identities resolve to exact contiguous source slices; revise `P_line-window` to define section-relative windows; retain `P_direct-retrieval-skips-index`.
- `contract-003-api-sokf`: revise `P_edit-exact-atomic` and mutation behavior to cover Pi-compatible argument preparation, exact-then-normalized matching, original-snapshot uniqueness and overlap checks, byte preservation, concise policy errors, and physical-path queue identity.
- `contract-003-api-sokf`: revise `P_write-whole-path-create` and mutation behavior for parent creation, queueing, cancellation, and concise adapter results while retaining agent-safe repair and validation.
- `contract-003-api-sokf`: clarify that `P_mutation-result-shape` is the MCP lifecycle envelope consumed internally by adapters and does not replace Pi's model-visible built-in result shape.

## ADR decisions

- none. The implementation follows the existing thin-adapter architecture and treats Pi's installed factories and exported helpers as the executable specification.

## Source and interface changes

Add a narrow SOKF resolution/source operation that yields the canonical repository-relative physical path, exact source, existence, and authority metadata. Refactor `.pi/extensions/sokf.ts` around Pi's `createReadTool()`, `createEditTool()`, `createWriteTool()`, diff helpers, and mutation queue behavior, overriding only resolution and agent-safe persistence. Narrow `.pi/extensions/sokf-mcp.ts` and the Rust MCP/mutation boundary as needed so lifecycle details remain extension-side and generated targets can be rejected before mutation.

The materialized file-tool schemas, metadata, renderers, result content, and details remain Pi-compatible. The MCP lifecycle envelope may remain available to non-Pi clients, but the Pi adapter returns built-in result shapes.

## Knowledge changes

Update `contract-003-api-sokf`, `architecture`, `software-components`, and `development-commands` after implementation so they describe exact-source identity routing, section-source slices, Pi-compatible adapter results, shared physical queueing, and bounded repair feedback. Update the canonical SOKF authoring skill and every shipped/materialized copy only where routing guidance changes. Remove or supersede the repository-root proposal document once its approved requirements are represented by this plan and current knowledge.

## Documentation changes

Canonical knowledge and owned Pi skill/extension copies are required. User-facing README and CLI help are not triggered because no user CLI command or released configuration surface changes. Each owning work block runs generation or copy checks for its affected surfaces.

- Final verification: `cargo fmt --check`.
- Final verification: `cargo clippy --workspace --all-targets -- -D warnings`.
- Final verification: `cargo nextest run --workspace`.
- Final verification: `cargo test --workspace --doc`.
- Final verification: `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps`.
- Final verification: `npm test`.
- Final verification: `npm run test:scripts`.
- Final verification: `npm run check:blueprint`.
- Final verification: `npm run check:validate`.

## Work blocks

### Block 1: Characterize Pi and SOKF file-tool contracts

- [ ] Done.
- Dependencies: none.
- Areas: `scripts/test/fixtures/sokf-pi-adapter-smoke.ts`, `scripts/test/sokf-pi-adapter.test.mjs`, and focused adapter test helpers.
- Outcome: a paired harness compares built-in and SOKF-routed content, details, errors, filesystem bytes, renderer inputs, cancellation, and one-line-edit result size while normalizing only unavoidable displayed target paths.
- Verification: `node --test scripts/test/sokf-pi-adapter.test.mjs`.
- Tests: paired cases cover `contract-003-api-sokf` properties `P_coding-read`, `P_concept-address`, `P_line-window`, `P_mutation-precondition-error`, `P_edit-exact-atomic`, and `P_write-whole-path-create`, including truncation, BOM, CRLF, Unicode, normalized matching, duplicates, overlap, disjoint edits, creation, overwrite, nested parents, abort, and oversized-content rejection.
- Structural evidence: harness execution uses Pi's installed tool factories as one side of every comparison and makes no model call.
- Documentation: none; this block establishes executable characterization.

### Block 2: Resolve identities to exact source with read parity

- [ ] Done.
- Dependencies: Block 1.
- Areas: `crates/lib/superdev-core/src/sokf/mcp.rs`, SOKF source resolution modules, `.pi/extensions/sokf.ts`, and MCP tests.
- Outcome: full concept identities read exact mutation-ready source; section identities read exact contiguous source slices with section-relative pagination; built-in truncation, errors, cancellation, details, metadata, and rendering are preserved.
- Verification: `cargo test -p superdev-core --test mcp_tools && node --test scripts/test/sokf-pi-adapter.test.mjs`.
- Tests: exact-source and section-slice cases cover `contract-003-api-sokf` properties `P_concept-address`, `P_line-window`, `P_direct-retrieval-skips-index`, `P_parse-error-quoted`, and `P_coding-read`; copied text including frontmatter succeeds as later edit input.
- Structural evidence: read delegation uses Pi's factory behavior rather than a local pagination or truncation implementation.
- Documentation: defer contract and architecture updates to Block 6 after behavior stabilizes.

### Block 3: Preserve Pi edit semantics through agent-safe mutation

- [ ] Done.
- Dependencies: Blocks 1 and 2.
- Areas: `.pi/extensions/sokf.ts`, `.pi/extensions/sokf-mcp.ts`, `crates/lib/superdev-core/src/sokf/mutation.rs`, MCP adapter code, and focused Rust/JavaScript tests.
- Outcome: virtual and physical edits use one physical queue key, Pi-compatible preparation and matching, byte-preserving persistence, concise success content, compact built-in details, and concise SOKF policy errors; mutation envelopes and whole-file patches remain outside model content.
- Verification: `cargo test -p superdev-core sokf && cargo test -p superdev-core --test mcp_tools && node --test scripts/test/sokf-pi-adapter.test.mjs`.
- Tests: paired edit cases cover `contract-003-api-sokf` properties `P_mutation-agent-safe`, `P_mutation-precondition-error`, `P_edit-exact-atomic`, `P_mutation-contained`, `P_mutation-repair-validation`, and `P_applied-invalid-result`.
- Structural evidence: Pi's exported matching/diff facilities produce edit results; no adapter function emits a whole-file diff into tool content or recommends write fallback.
- Documentation: none beyond source comments required to distinguish MCP lifecycle data from Pi results.

### Block 4: Preserve Pi write semantics and bounded lifecycle reporting

- [ ] Done.
- Dependencies: Blocks 1 and 2.
- Areas: `.pi/extensions/sokf.ts`, `.pi/extensions/sokf-mcp.ts`, Rust write/resolution paths, final-validation feedback, and focused tests.
- Outcome: SOKF writes match Pi's parent creation, queueing, cancellation, errors, concise content, undefined details, and renderer while repair/refile/validation diagnostics remain bounded and extension-side.
- Verification: `cargo test -p superdev-core --test mcp_tools && node --test scripts/test/sokf-mcp-client.test.mjs && node --test scripts/test/sokf-pi-adapter.test.mjs`.
- Tests: write and lifecycle cases cover `contract-003-api-sokf` properties `P_write-whole-path-create`, `P_mutation-contained`, `P_mutation-repair-validation`, `P_applied-invalid-result`, and `P_mutation-result-shape` without leaking its envelope into Pi content.
- Structural evidence: virtual and physical aliases resolve before queueing; successful Pi write details are undefined even when repair touches additional files.
- Documentation: none; current-state documentation is consolidated in Block 6.

### Block 5: Enforce generated ownership and simplify prompt metadata

- [ ] Done.
- Dependencies: Blocks 2, 3, and 4.
- Areas: SOKF generated-region policy, `.pi/extensions/sokf.ts`, prompt metadata tests, `.pi/skills/sokf-authoring/SKILL.md`, `pack/pi/` shipped copies, and lock/materialization entries.
- Outcome: generated-projection mutations fail before persistence with the authoritative source path; file tools inherit Pi metadata and rendering; discovery wording belongs to `sokf_search` and `sokf_graph`; only three concise tool-prefixed routing rules remain.
- Verification: `node --test scripts/test/sokf-pi-adapter.test.mjs && npm run check:blueprint`.
- Tests: direct in-process metadata assertions cover registered snippets and guidelines without a Pi process, MCP server, repository fixture, filesystem setup, model call, or broad prompt snapshot.
- Structural evidence: each known-ID lookup, existing-concept edit, and new-concept creation rule appears once; no rule duplicates inherited Pi guidance or recommends whole-file recovery.
- Documentation: update source and shipped SOKF authoring skills together and verify blueprint parity.

### Block 6: Remove duplicate implementations and publish current behavior

- [ ] Done.
- Dependencies: Blocks 3, 4, and 5.
- Areas: obsolete adapter diff/result helpers, `whole_file_diff` consumers, `contract-003-api-sokf`, `architecture`, `software-components`, `development-commands`, and `SOKF-EDIT-RELIABILITY-PLAN.md`.
- Outcome: duplicate Pi behavior and unbounded model-visible mutation reporting are removed or narrowly retained for non-Pi consumers; canonical contracts and architecture describe the implemented behavior; the superseded external proposal no longer competes with this plan.
- Verification: `cargo test -p superdev-core sokf && node --test scripts/test/sokf-pi-adapter.test.mjs && cargo run --quiet -- validate --fix && cargo run --quiet -- validate`.
- Tests: final focused suites retain coverage of all changed `contract-003-api-sokf` properties and direct prompt metadata assertions.
- Structural evidence: repository search finds no adapter-owned replacement for Pi diff/rendering algorithms and no normal file-tool result populated from the MCP mutation envelope.
- Documentation: canonical-knowledge; generate includes with `cargo run --quiet -- validate --fix`, then verify with `cargo run --quiet -- validate`.

## Build state

Current block: 1. Attempts: 0. Final corrections: 0. Blocker: none.

## Implementation decisions

none.

## Follow-up issues

none.

## Completion evidence

Scope review and approval are pending; BUILD evidence has not started.

<!-- sokf:links -->
[sokf:issue-077-sokf-file-tool-parity]: /knowledge/issues/open/issue-077-sokf-file-tool-parity.md
