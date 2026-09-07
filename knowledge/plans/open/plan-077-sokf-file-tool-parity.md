---
type: Plan
id: plan-077-sokf-file-tool-parity
title: "SOKF-routed file tools match Pi's built-in contracts"
description: "Route SOKF identities through Pi-compatible read, edit, and write behavior while preserving agent-safe repair, validation, and bounded diagnostics."
lifecycle: open
phase: scope
branch: work/077-sokf-file-tool-parity
links:
  - rel: implements
    to: issue-077-sokf-file-tool-parity
---

# Plan: SOKF-routed file tools match Pi's built-in contracts

## Goal and boundaries

Implement [SOKF-routed file tools match Pi's built-in contracts][sokf:issue-077-sokf-file-tool-parity]. A physical file path and its `sokf:<id>` alias must expose the same Pi 0.85.1 file-tool behavior. SOKF adds identity routing, agent-safe policy, repair, refiling, and validation only.

The installed Pi factories, operations interfaces, result types, renderers, argument preparation, `generateDiffString()`, `generateUnifiedPatch()`, and `withFileMutationQueue()` are the executable reference. The adapter must delegate to those interfaces or use a narrow adapter proven byte-for-byte against them. The SOKF Model Context Protocol (MCP) contract is unreleased, so this plan deliberately changes its mixed retrieval/file-read boundary instead of preserving `sokf_read` compatibility. MCP changes must remain limited to source routing, semantic retrieval separation, parity-safe mutation transport, and bounded diagnostics. The work excludes approximate copies of Pi algorithms, a semantic result dialect in Pi's `read` slot, automatic whole-file recovery after an edit mismatch, weakened SOKF policy, and a new extension packaging model.

## Requirements

The implementation must satisfy the following settled requirements.

- `read path="sokf:<id>"` must resolve the identity to its canonical repository-relative path and return the exact UTF-8 source bytes through Pi's built-in read behavior.
- Routed reads must preserve 1-indexed `offset` and `limit`, 2,000-line and 50 KB head truncation, continuation messages, `ReadToolDetails.truncation`, access errors, out-of-range errors, cancellation, metadata, and rendering.
- Pi's file `read` must reject `sokf:` overview and section-qualified addresses with concise guidance.
- MCP must remove the mixed-purpose `sokf_read` tool. A new `sokf_resolve_source` tool must accept an unqualified `sokf:<id>` or a contained physical knowledge path and return canonical path, existence, and generated-authority metadata without rendering semantic content.
- MCP must expose semantic overview, rendered-concept, and section retrieval as `sokf_retrieve`. `sokf_retrieve` must retain the current semantic output and line-window behavior for those address forms, refuse physical file paths, and require no `sokf_read` compatibility alias or deprecation period.
- Routed edits must preserve Pi's current `edits[]` schema, legacy argument preparation, non-empty validation, exact-then-normalized matching, one-snapshot uniqueness and overlap checks, byte-order mark and line-ending preservation, cancellation, and error wording where SOKF policy does not intervene.
- The routed edit must resolve the canonical target before invoking Pi's edit factory. Pi's `EditOperations` must read the exact source and submit the computed final content through an agent-safe compare-and-swap mutation carrying the exact original content. The mutation must refuse stale original content without changing any file.
- A successful routed edit must return `Successfully replaced N block(s) in PATH.` and Pi-compatible `EditToolDetails` with the compact display diff, standard unified patch, and `firstChangedLine`.
- Routed writes must preserve `{ path, content }`, recursive parent creation, cancellation, errors, `Successfully wrote to PATH`, `details: undefined`, metadata, and rendering.
- An abort observed before MCP persistence dispatch must produce Pi's `Operation aborted` error, leave the target bytes unchanged, make no mutation-diagnostic entry, and schedule no final validation.
- Once an edit or write dispatches MCP persistence, the adapter must stop forwarding the tool abort signal to that mutation request. The adapter must keep the canonical target queue locked until the request settles.
- When a shielded mutation returns `applied: true`, the adapter must record the changed final path, bounded repair or refiling diagnostics, and pending final-validation state before its custom persistence operation resolves. If Pi observes the abort after that resolution, the file tool must throw `Operation aborted`, emit no success content or details, and still run final validation at the next `turn_end`.
- When shielded persistence fails without `applied: true`, the adapter must propagate the mutation failure, create no applied-mutation state, and schedule no final validation. The tool abort must not replace that mutation failure.
- Virtual and physical aliases must use the same canonical physical target as the `withFileMutationQueue()` key. The queue must cover each complete read-modify-write window, including shielded post-dispatch persistence.
- Agent-safe mutations must continue to preserve `id` and existing `verified` bytes, reject stamped fields, remain contained under `knowledge/`, and run repair, refiling, and validation.
- Edits intersecting generated projections must fail before mutation and name the authoritative source. The adapter must never recommend or perform an automatic whole-file fallback after a targeted-edit failure.
- MCP `sokf_edit` must act only as the compare-and-swap source transport described below. The Pi `edit` schema must remain the only model-facing file-edit dialect. MCP mutation state and bounded diagnostics must remain available extension-side, while mutation JSON, full repair reports, validation dumps, and duplicate or whole-file patches must not enter file-tool model content.
- Each repair or refiling diagnostic must use a non-context session entry capped at 200 lines or 8 KiB, whichever occurs first. Each final validation follow-up must contain only actionable findings, must omit patches, and must use the same 200-line and 8 KiB caps. Truncated diagnostics must direct the agent to `superdev validate` for the complete report.
- `read`, `edit`, and `write` must retain Pi's built-in prompt snippets, baseline guidelines, descriptions, schemas, and renderers. Prompt metadata must add only these flat SOKF rules: `sokf_search: use it when no concept ID is known; otherwise read sokf:<id>.`, `edit: use sokf:<id> for an existing knowledge concept; section addresses are read-only.`, and `write: create a knowledge concept at its physical knowledge/...md path.`
- `sokf_search` and `sokf_graph` must own the two added Available-tools descriptions: `Search canonical project knowledge by meaning` and `Follow relationships between canonical knowledge concepts`.
- A paired harness must compare built-in and routed content, details, thrown errors, resulting bytes, and renderer inputs exactly. The harness may normalize only the unavoidable physical-versus-virtual target spelling.
- The harness must cover success, failure, offset, limit, both truncation limits, a single oversized line, byte-order marks, CRLF, Unicode, normalized matching, duplicate matches, overlap, multiple disjoint edits, creation, overwrite, and nested parent creation. For both edit and write, deterministic barriers must cover abort before persistence dispatch, abort after persistence returns `applied: true`, and abort concurrent with failed persistence.
- A copied routed-read excerpt, including frontmatter, must work unchanged as edit `oldText`. A one-line edit to a large concept must produce the same model-visible result bytes as the paired built-in edit after path normalization.
- Existing shipped and materialized copies must remain synchronized. Repository inspection found only `.pi/skills/sokf-authoring/SKILL.md` mirrored under `pack/pi/skills/`; the project-local SOKF adapter has no pack copy or `.superdev/lock.toml` claim.
- BUILD must remove `/SOKF-EDIT-RELIABILITY-PLAN.md` only after the canonical issue, plan, contracts, ADR, tests, and documentation capture its requirements.

## Contract changes

- `contract-003-api-sokf`: replace the mixed-purpose `sokf_read` definition with materialized `sokf_resolve_source` and `sokf_retrieve` definitions. The resolver result carries the canonical repository-relative physical path, existence, and generated-region authority metadata; Pi reads exact UTF-8 source through its own file operation. Add `P_source-resolution`, `AC_source-identity`, `AC_source-contained`, `AC_source-section-refused`, `P_semantic-retrieve`, `AC_semantic-addresses`, `AC_semantic-physical-refused`, `P_no-read-alias`, `P_edit-compare-and-swap`, `AC_edit-stale-source-refused`, and `P_mutation-diagnostics-bounded`. Replace the MCP `sokf_edit` request's exact-edit array with the narrow `{ path, expectedContent, content }` source-replacement transport, while retaining `sokf_edit` as the operation name and mandatory agent-safe policy. Remove requested and repair patches from MCP mutation results; retain applied state, validation state, resolved and final paths, changed-path summaries, and bounded findings for extension-side lifecycle handling. Remove or revise the current `P_coding-read`, `P_overview-address`, `P_concept-address`, `P_physical-contained`, `P_physical-refused`, `P_line-window`, `P_direct-retrieval-skips-index`, `P_parse-error-quoted`, `P_edit-exact-atomic`, `P_mutation-result-shape`, and overview-limit promises to match the split. Retain all unaffected search, graph, transport, authentication, lifecycle, and safety promises.
- `contract-012-api-sokf-pi-file-tools`: add the public Pi adapter contract from a source region in `.pi/extensions/sokf.ts`. Add `P_read-parity`, `P_edit-parity`, `P_write-parity`, `P_alias-queue`, `P_agent-safe-routing`, `P_persistence-abort-boundary`, `P_context-shape`, `P_generated-authority`, and `P_prompt-ownership`, with nested acceptance criteria for every requirement above. `P_persistence-abort-boundary` must include `AC_pre-persistence-abort`, `AC_edit-post-persistence-abort`, `AC_write-post-persistence-abort`, `AC_aborted-apply-validation`, and `AC_failed-persistence-not-dirty`. Mark unbuilt behavior `PENDING(plan-077/block-N)` until its implementing block passes, then remove every pending marker before acceptance.

## ADR decisions

- `adr-053-sokf-file-tools-delegate-to-pi`: record that SOKF resolves identities before execution, delegates file semantics and presentation to Pi factories and exported helpers, queues by canonical physical target, commits edits through an agent-safe compare-and-swap source transport, and keeps lifecycle diagnostics outside model-visible file-tool content. Define persistence dispatch as the cancellation boundary: pre-dispatch aborts retain Pi behavior, while dispatched mutations settle under the queue so an applied result is recorded before Pi reports a post-persistence abort. Record the deliberate unreleased MCP break that replaces `sokf_read` with `sokf_resolve_source` and `sokf_retrieve`. Reject compatibility aliases, independent emulation, rendered virtual file reads, mutation-envelope file-tool results, and releasing the queue while persistence can still complete.

## Source and interface changes

`crates/lib/superdev-core/src/sokf/mcp.rs` must replace `sokf_read` with `sokf_resolve_source` and `sokf_retrieve`. Resolution must remain a narrow machine operation for unqualified identities and contained physical paths. Retrieval must retain existing overview, rendered-concept, section, line-window, index-loading, and parse-error behavior while refusing physical paths. No compatibility alias remains. `crates/lib/superdev-core/src/sokf/mutation.rs` must expose generated-region authority and an agent-safe source replacement that compares `expectedContent` byte-for-byte with the current UTF-8 source before policy checks or persistence. MCP `sokf_edit` must use that source replacement so Pi owns edit matching. MCP mutation results must replace patch-bearing changes with changed-path summaries and bounded findings. BUILD must remove `whole_file_diff()` after no MCP result references it.

`.pi/extensions/sokf.ts` must resolve before delegation and invoke complete Pi tool definitions so schemas, argument preparation, constrained sampling, metadata, result construction, and renderers remain Pi-owned. Routed edits must pass the canonical physical path to the Pi factory and provide custom `EditOperations`: `readFile` captures exact original bytes, and `writeFile` submits the original and computed replacement to MCP `sokf_edit`. Routed writes must pass the canonical physical path to the Pi factory and provide custom `WriteOperations` that preserve recursive parent creation while submitting complete content through agent-safe MCP persistence. Both factories must therefore queue the complete operation on the canonical physical target. Resolution and all pre-dispatch work must remain abortable. Each custom persistence operation must omit the tool signal from the dispatched MCP mutation, await the complete result under the queue, and capture applied state before resolving to the Pi factory. Pi's factory must then perform its normal post-write abort check. An applied edit or write that Pi reports as `Operation aborted` must retain the changed bytes, emit no successful tool result, persist bounded lifecycle diagnostics, and trigger final validation at the next `turn_end`. A failed persistence request must remain an error without applied state or final validation. The adapter must normalize only the displayed virtual-versus-physical target spelling. It must keep mutation lifecycle data extension-side and persist diagnostics capped at 200 lines or 8 KiB, including truncation guidance, through `pi.appendEntry()`. `.pi/extensions/sokf-mcp.ts` must adopt the renamed retrieval, typed source-resolution, compare-and-swap edit, and narrowed mutation-result transports. Its process reuse, restart, abort, and shutdown contracts remain unchanged.

The contract Definition blocks must materialize the marked Rust and TypeScript source declarations through `cargo run -- validate --fix`. No command-line interface, generated CLI reference, dependency, `package.json`, or `package-lock.json` change is expected. The existing skill source and `pack/pi/skills/sokf-authoring/SKILL.md` mirror must move together, and `cargo run -- sync` must update `.superdev/lock.toml` when their bytes change. The adapter remains project-local because the current pack declares no SOKF adapter item.

Primary implementation and evidence files are `.pi/extensions/sokf.ts`, `.pi/extensions/sokf-mcp.ts`, `crates/lib/superdev-core/src/sokf/mcp.rs`, `crates/lib/superdev-core/src/sokf/mutation.rs`, `crates/lib/superdev-core/tests/mcp_tools.rs`, `scripts/test/fixtures/sokf-mcp-fake.mjs`, `scripts/test/fixtures/sokf-pi-adapter-smoke.ts`, `scripts/test/sokf-pi-adapter.test.mjs`, `scripts/test/sokf-mcp-client.test.mjs`, `.pi/skills/sokf-authoring/SKILL.md`, `pack/pi/skills/sokf-authoring/SKILL.md`, and `.superdev/lock.toml`.

## Knowledge changes

Add `contract-012-api-sokf-pi-file-tools` and `adr-053-sokf-file-tools-delegate-to-pi`. Update `contract-003-api-sokf`, `architecture`, `software-components`, `testing-strategy`, `development-commands`, and `error-handling` after implementation. Update `idea-012-sokf-mutations-survive-validation-failures` with the settled post-persistence cancellation behavior, and link `issue-077-sokf-file-tool-parity` to that originating idea with `references`. The contract and architecture must describe `sokf_resolve_source`, `sokf_retrieve`, the removal of `sokf_read`, and the compare-and-swap `sokf_edit` schema. Keep the idea, issue, and this plan current through BUILD evidence and acceptance. Let validation regenerate contract, ADR, idea, issue, and plan indexes and all source include and link blocks.

Remove `/SOKF-EDIT-RELIABILITY-PLAN.md` in the final documentation block. The canonical issue, plan, contracts, and ADR become the durable record of its parity, architecture, work plan, files, verification, and acceptance intent.

## Documentation changes

The documentation map triggers the following surfaces.

- `readme`: update `/README.md` because Pi file-tool behavior is user-visible and the MCP tool names, tool count, and retrieval guidance change. Verify with `npm run check:docs`.
- `contributor-guide`: update `/CONTRIBUTING.md` to name the paired adapter harness within `npm run test:scripts`. No development command changes are required. Verify with `npm run check:docs`.
- `canonical-knowledge`: update the contracts, ADR, architecture, components, tests, error handling, and command guidance listed above. Generate with `cargo run -- validate --fix` and verify with `npm run check:validate`.
- `changelog`: add the parity change under `/CHANGELOG.md` `[Unreleased]`. Verify with `npm run check:docs`.
- `cli-reference`: not triggered because command, argument, help, exit-code, stream, and workflow protocol shapes remain unchanged.
- `documentation-site`: not triggered because the repository still has no site source or renderer. Confirm with `npm run check:docs`.
- Packaging copies: update the live and pack copies of the authoring skill together, run `cargo run -- sync`, and verify `npm run check:blueprint`. Do not create a new packaged adapter item.

## Work blocks

### Block 1: Split MCP retrieval and deliver read parity

- [ ] Done.
- Dependencies: none.
- Areas: `contract-003-api-sokf`, `contract-012-api-sokf-pi-file-tools`, `adr-053-sokf-file-tools-delegate-to-pi`, `crates/lib/superdev-core/src/sokf/mcp.rs`, `.pi/extensions/sokf.ts`, `.pi/extensions/sokf-mcp.ts`, `crates/lib/superdev-core/tests/mcp_tools.rs`, and the paired JS harness.
- Outcome: `sokf_resolve_source` yields a canonical physical target for Pi's exact-source read, `sokf_retrieve` owns semantic overview and section behavior, `sokf_read` is absent, and routed Pi read behavior matches the paired built-in.
- Verification: `cargo test -p superdev-core sokf && cargo test -p superdev-core --test mcp_tools && node --test scripts/test/sokf-mcp-client.test.mjs && node --test scripts/test/sokf-pi-adapter.test.mjs`.
- Tests: paired read cases cover `contract-012-api-sokf-pi-file-tools P_read-parity`; MCP cases cover `contract-003-api-sokf P_source-resolution`, `AC_source-identity`, `AC_source-contained`, `AC_source-section-refused`, `P_semantic-retrieve`, `AC_semantic-addresses`, `AC_semantic-physical-refused`, and `P_no-read-alias`.
- Structural evidence: the MCP tool-list assertion contains `sokf_resolve_source` and `sokf_retrieve` but not `sokf_read`; `cargo run -- validate` proves both contract Definitions match their marked source regions and every Block 1 pending marker is removed.
- Documentation: create the ADR and contracts, run `cargo run -- validate --fix`, then run `npm run check:validate`.

### Block 2: Deliver edit parity and generated-region safety

- [ ] Done.
- Dependencies: Block 1.
- Areas: `contract-003-api-sokf`, `.pi/extensions/sokf.ts`, `.pi/extensions/sokf-mcp.ts`, `crates/lib/superdev-core/src/sokf/mcp.rs`, `crates/lib/superdev-core/src/sokf/mutation.rs`, `crates/lib/superdev-core/tests/mcp_tools.rs`, `scripts/test/fixtures/sokf-pi-adapter-smoke.ts`, `scripts/test/sokf-mcp-client.test.mjs`, and `scripts/test/sokf-pi-adapter.test.mjs`.
- Outcome: routed edits use Pi-compatible argument preparation, matching, preservation, errors, details, rendering, and target queueing while compare-and-swap persistence rejects stale, protected, or generated changes before mutation.
- Verification: `cargo test -p superdev-core sokf && cargo test -p superdev-core --test mcp_tools && node --test scripts/test/sokf-mcp-client.test.mjs && node --test scripts/test/sokf-pi-adapter.test.mjs`.
- Tests: paired edit cases cover `contract-012-api-sokf-pi-file-tools P_edit-parity`, `P_alias-queue`, `P_agent-safe-routing`, `P_generated-authority`, `P_persistence-abort-boundary AC_pre-persistence-abort`, `AC_edit-post-persistence-abort`, and `AC_failed-persistence-not-dirty` across stale, duplicate, overlap, disjoint, BOM, CRLF, Unicode, normalized, policy, repair, and refiling cases. MCP tests cover every changed `contract-003-api-sokf` mutation promise, compare `expectedContent` byte-for-byte, and prove an atomic stale-source refusal.
- Structural evidence: a deterministic persistence barrier proves an edit abort before dispatch makes no MCP call and changes no target bytes. A second barrier pauses after dispatch, aborts the tool, proves a physical-alias edit remains queued, releases an `applied: true` response, and asserts changed bytes plus the exact `Operation aborted` error with no success result. A failed-response variant asserts the mutation error wins and no applied state exists. Another test copies frontmatter and body text from routed read into `oldText`; another proves a generated-projection rejection leaves every file byte-identical and names the authoritative source.
- Documentation: update pending contract criteria and run `cargo run -- validate --fix` followed by `npm run check:validate`.

### Block 3: Deliver write parity and bounded lifecycle reporting

- [ ] Done.
- Dependencies: Blocks 1 and 2.
- Areas: `.pi/extensions/sokf.ts`, `.pi/extensions/sokf-mcp.ts`, `crates/lib/superdev-core/src/sokf/mutation.rs`, `scripts/test/fixtures/sokf-mcp-fake.mjs`, `scripts/test/fixtures/sokf-pi-adapter-smoke.ts`, `scripts/test/sokf-mcp-client.test.mjs`, and the paired JS harness.
- Outcome: routed writes match Pi, pre-dispatch aborts do not mutate, dispatched edit and write persistence settles under the queue, applied state survives Pi's post-persistence abort error, mutation details remain extension-side, repair and refiling diagnostics use bounded non-context entries, and final validation follow-ups contain bounded actionable findings without patches.
- Verification: `cargo test -p superdev-core sokf && node --test scripts/test/sokf-mcp-client.test.mjs && node --test scripts/test/sokf-pi-adapter.test.mjs`.
- Tests: paired creation, overwrite, nested-parent, error, alias-queue, policy-rejection, repair, and refiling cases cover `contract-012-api-sokf-pi-file-tools P_write-parity`, `P_agent-safe-routing`, `P_context-shape`, `P_persistence-abort-boundary AC_pre-persistence-abort`, `AC_write-post-persistence-abort`, `AC_aborted-apply-validation`, and `AC_failed-persistence-not-dirty`, plus `contract-003-api-sokf P_mutation-diagnostics-bounded`. Deterministic edit and write cases each abort before dispatch and after an `applied: true` persistence barrier. Both post-persistence cases assert retained bytes, no successful tool result, bounded lifecycle entries, and exactly one validation invocation at the next `turn_end`. Failed edit and write persistence cases assert no validation invocation. Session cases prove every complete lifecycle entry and validation follow-up, including truncation guidance, stops at 200 lines or 8 KiB and directs truncated validation output to `superdev validate`.
- Structural evidence: the persistence-barrier cases prove the abort signal does not terminate a dispatched mutation and a queued physical or virtual alias cannot start before that mutation settles. A validation spy proves rejected post-persistence edit and write calls both set pending validation, while pre-dispatch aborts and failed persistence do not. A one-line edit of a large concept has built-in-equivalent model-visible bytes after path normalization; session inspection finds no mutation JSON, repair patch, or validation dump in file-tool content.
- Documentation: update pending contract criteria and run `cargo run -- validate --fix` followed by `npm run check:validate`.

### Block 4: Minimize prompt metadata and synchronize authoring guidance

- [ ] Done.
- Dependencies: Blocks 1, 2, and 3.
- Areas: `.pi/extensions/sokf.ts`, `.pi/skills/sokf-authoring/SKILL.md`, `pack/pi/skills/sokf-authoring/SKILL.md`, `.superdev/lock.toml`, and prompt metadata assertions in the paired JS harness.
- Outcome: file tools retain Pi's metadata, only the three approved flat SOKF routing rules are added, discovery wording belongs to `sokf_search` and `sokf_graph`, and skill copies describe exact-source editing and authoritative generated sources consistently.
- Verification: `node --test scripts/test/sokf-pi-adapter.test.mjs && cargo run -- sync && npm run check:blueprint && npm run check:validate`.
- Tests: fast in-process metadata assertions cover `contract-012-api-sokf-pi-file-tools P_prompt-ownership`; they prove each routing decision appears once and no rule recommends whole-file fallback.
- Structural evidence: the test directly inspects registered snippets and guideline arrays. It may snapshot only the generated Available-tools and Guidelines sections when generation needs no Pi process, MCP server, repository fixture, filesystem setup, or model call.
- Documentation: update both skill copies, refresh the lock through sync, and verify the second `cargo run -- sync --dry-run` reports no drift.

### Block 5: Update documentation, remove the proposal, and verify the repository

- [ ] Done.
- Dependencies: Blocks 1, 2, 3, and 4.
- Areas: `/README.md`, `/CONTRIBUTING.md`, `/CHANGELOG.md`, `/SOKF-EDIT-RELIABILITY-PLAN.md`, `idea-012-sokf-mutations-survive-validation-failures`, `issue-077-sokf-file-tool-parity`, `architecture`, `software-components`, `testing-strategy`, `development-commands`, `error-handling`, all changed contracts and ADRs, and generated indexes.
- Outcome: public and canonical documentation describe the implemented behavior, the obsolete root proposal is gone, every pending contract marker is removed, and focused plus complete suites pass.
- Verification: `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo nextest run --workspace && cargo test --doc --workspace && RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps && npm run test:launcher && npm run test:scripts && npm run verify-version && npm run check:docs && npm run check:validate && npm run check:blueprint && npm run coverage:check && cargo deny check licenses bans sources`.
- Tests: `cargo test -p superdev-core sokf`, `cargo test -p superdev-core --test mcp_tools`, `node --test scripts/test/sokf-pi-adapter.test.mjs`, and `node --test scripts/test/sokf-mcp-client.test.mjs` remain named focused evidence for all parity contract criteria.
- Structural evidence: manually compare paired physical and `sokf:<id>` calls for read, edit, and write in Pi. Inspect collapsed and expanded rendering plus session JSONL, and record that only normalized path spelling differs. List MCP tools and verify the documented `sokf_read` removal and `sokf_resolve_source`/`sokf_retrieve` split.
- Documentation: run `cargo run -- validate --fix` twice, require a clean second pass, then run `npm run check:docs`, `npm run check:validate`, and `npm run check:blueprint`.

## Build state

Current block: 1. Attempts: 0. Final corrections: 0. Blocker: scope requirements review and human approval pending.

## Implementation decisions

none. BUILD records only local choices that do not change the approved contracts or ADR.

## Follow-up issues

none. Any unrelated packaging redesign or change to semantic ranking, rendered retrieval content, MCP transport lifecycle, search, or graph behavior must use a separate issue.

## Completion evidence

Scope requirements review and explicit human approval are pending. The parent workflow must publish the approved knowledge-only scope as the immutable SCOPE checkpoint before BUILD starts.

<!-- sokf:links -->
[sokf:issue-077-sokf-file-tool-parity]: /knowledge/issues/open/issue-077-sokf-file-tool-parity.md
