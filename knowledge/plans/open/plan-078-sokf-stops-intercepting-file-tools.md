---
type: Plan
id: plan-078-sokf-stops-intercepting-file-tools
title: SOKF stops intercepting the file tools
description: Remove routed read, edit, and write, serve search, graph, and overview alone, and make one repair-and-validate at turn end the single point where knowledge becomes consistent.
lifecycle: open
phase: scope
branch: work/095-sokf-stops-intercepting-file-tools
links:
  - rel: implements
    to: issue-095-sokf-stops-intercepting-file-tools
---

# Plan: SOKF stops intercepting the file tools

## Goal and boundaries

Implement [SOKF stops intercepting the file tools][sokf:issue-095-sokf-stops-intercepting-file-tools].
A Pi session reads and edits canonical knowledge with Pi's own `read`, `edit`,
and `write` on physical paths, SOKF MCP serves only what a file tool cannot do,
and the knowledge is repaired and validated once per turn.

This plan implements
[adr-055][sokf:adr-055-sokf-does-not-intercept-file-tools], which supersedes the
now-deprecated `adr-053`. It reverts most of
[issue-077][sokf:issue-077-sokf-file-tool-parity], merged earlier on the same
day: routed reads returning exact source is reached more cheaply by not routing,
because an unrouted read returns exact source by opening the file.

The work is ordered so that no intermediate state is broken. Additive changes
land first, the turn-end mechanism that replaces mutation-time repair lands
before the repair it replaces is removed, and the MCP operations are removed only
after the extension has stopped calling them.

The work excludes the `superdev sokf edit` and `superdev sokf write` commands,
`SokfService::edit` and `write` as library calls, and the inline repair they
perform. Their callers are deterministic rather than agents: the workflow
transitions in `crates/app/superdev/src/workflow_cli/records.rs` gate on
`mutation.validation` and depend on refiling, and
`contract-002-cli-superdev P_sokf-mutation-repairs` settles the behaviour with no
pending promise. It also excludes semantic ranking, index lifecycle, MCP
transport, search behaviour, what validation checks or repairs, and any
replacement for the mutation-time policy that routing enforced.

## Requirements

The implementation must satisfy the following settled requirements.

- MCP must serve exactly three tools: `sokf_search`, `sokf_graph`, and
  `sokf_overview`. `sokf_resolve_source`, `sokf_retrieve`, `sokf_edit`, and
  `sokf_write` must be absent from the tool list, and `get_info()` must name only
  served tools.
- `sokf_overview` must accept no required argument and return the knowledge name,
  concept count, tree, index state, and capped validation warnings — the output
  `SokfService::overview()` already produces. It must carry no rendered concept,
  no section addressing, and no line window.
- `sokf_graph` must return each named concept's repository-relative path beside
  its identity and description, so a traversal reaches a file without a second
  lookup, as `sokf_search` results already allow.
- The Pi extension must register `sokf_search`, `sokf_graph`, and
  `sokf_overview`, and must register no `read`, `edit`, or `write` tool. Pi's
  built-ins must reach the session unmodified, with Pi's own descriptions, prompt
  snippets, guidelines, schemas, renderers, and results.
- A `knowledge/` path must behave in every file tool exactly as any other path.
  No SOKF resolution, containment check, policy check, notice rewriting, or
  result dialect may apply to it.
- The extension must retain repository discovery for one purpose only: locating
  the MCP process per repository root. The ranked `.git`-then-`.superdev/config.toml`
  walk is retained for that purpose and must keep working from a nested working
  directory and a linked worktree.
- When a turn ends, the extension must run `superdev validate --fix` once,
  unconditionally, without first deciding whether the knowledge changed. A write
  through `bash`, a heredoc, `sed`, a patch, or `git checkout` reaches no tool
  the extension registers, so any change signal it could keep would be wrong on
  exactly the cases this issue exists to cover. An unconditional run needs no
  signal to be right.
- The unconditional run must leave the tree byte-identical and report
  `repaired: 0 file(s)` when the knowledge is already valid, so a turn that
  touched no knowledge changes nothing. It must cost one `validate --fix` over
  the canonical knowledge, with no second pass and no retry.
- Before sending any message, the extension must re-check its findings against
  the current working tree and report only those the tree still carries. A report
  that survives no finding must send no message.
- The first report for a pending sequence must send one visible triggering
  follow-up. A later report must be visible and non-triggering. Follow-up state
  must be session memory keyed by canonical repository root, and nothing may
  persist.
- Every visible message must carry concise paths, locations where available,
  messages, and corrective actions; must stop at 200 lines or 8 KiB; must omit
  patches and mutation envelopes; and must direct a truncated report to
  `superdev validate`.
- Repair, validation, and follow-up state must stay inside the active checkout,
  including a linked Git worktree.
- `superdev sokf edit`, `superdev sokf write`, and every workflow transition must
  behave exactly as they do today, including inline repair, refiling, and the
  `validation` gate. Their output must stay byte-compatible.
- `contract-012-api-sokf-pi-file-tools` must become `deprecated`, because the
  adapter interface it defines ceases to exist. Its Definition must keep
  materializing from source, so the removed registrations must survive as an
  archived file the contract includes, as `contract-009-interface-run-state`
  includes `/archive/claude-code/run.rs`. A Definition carrying no include fails
  the contract schema's `content: include` rule, so dropping the include without
  replacing it is not available.
- The surviving `.pi/extensions/sokf.ts` behaviour must be bound by a contract
  rather than by tests alone. `contract-003-api-sokf` binds the MCP server and
  `contract-011-interface-workflow` binds `.pi/extensions/superdev/`; neither
  reaches this file, so deprecating `contract-012` would leave the turn-end
  mechanism — the only thing guaranteeing that knowledge is consistent when a
  turn ends — with no promise a future change could break. A new
  `contract-013-interface-sokf-pi-extension` must bind it, and must land in
  Block 2 with the behaviour rather than in Block 3 with the removal.
- The routed promises on `contract-003-api-sokf` must be removed rather than left
  describing absent operations, and its retained promises must be accurate
  against the three served tools.
- `contract-003-api-sokf` must stop defining the mutation request types, because
  MCP accepts no mutation request. Those types must keep a contract, because
  `superdev sokf edit --request-json` and `sokf write --request-json` remain
  public callers of them, so `contract-002-cli-superdev` must define them.
- No pack copy or `.superdev/lock.toml` claim exists for `.pi/extensions/sokf.ts`,
  so `sync` will not overwrite it.
- The standing agent instruction must stop directing an agent at
  `read path="sokf:<id>"`, because no tool answers that address any more. Its
  source is `crates/lib/superdev-core/src/agent-instructions.md`, and
  `.agents/superdev.md` must be regenerated from that source by
  `cargo run -- sync` rather than hand-edited, so `npm run check:blueprint`
  stays clean.
- The `sokf-authoring` skill must stop describing the SOKF adapter, routed
  `edit path="sokf:<id>"`, mutation-tool repair of generated blocks,
  `applied: true`, and two automatic repair follow-ups. The `pack/pi/skills/`
  mirror must stay byte-identical to it.
- `evals/sokf/behavioral.json` must stop asserting tool calls no session can
  make. Seven scenarios name a `sokf:` path for `read` or `edit`, not three:
  `architecture-decision`, `direct-concept-reference`, `concept-edit`,
  `multi-step-knowledge-change`, `knowledge-producing-code-change`,
  `outward-facing-documentation`, and `unknown-concept-id`. The scenario roster,
  each sandbox, and the acceptance threshold stay unchanged.
- Six of those seven are path substitutions that keep their intent exactly:
  each names a physical `knowledge/` path where it names a `sokf:` address.
  `unknown-concept-id` is not, and must be re-aimed. It scores
  `near-miss-or-search-recovery` on a routed read of an unknown identity, and an
  unrouted `read path="sokf:command-router-policy"` is an ordinary missing-file
  read, so the behaviour it measures ceases to exist. It must instead name a
  missing `knowledge/` path and score recovery through `sokf_search`, keeping
  what the scenario protected — an agent that recovers semantically rather than
  scanning the filesystem — under an address that still resolves. That is the
  one intent change in this plan, and BUILD must not extend it to another
  scenario. `unknown-concept-id` is absent from `safetyCriticalScenarios`, so
  the safety-critical set is untouched.

## Contract changes

- `contract-003-api-sokf`: add `P_overview-tool` with `AC_overview-content`
  covering the knowledge name, concept count, tree, and index state, and
  `AC_overview-no-arguments` covering its empty request schema. Add
  `AC_graph-carries-paths` under the existing graph promise, requiring a
  repository-relative path beside each named concept. Retarget
  `P_overview-warning-cap` from the `sokf:` retrieval address to `sokf_overview`.
  Remove `P_routed-schemas` with all its criteria, `P_source-resolution`,
  `P_semantic-retrieve`, `P_no-read-alias`, `P_direct-retrieval-skips-index`,
  `P_mutation-agent-safe`, `P_edit-compare-and-swap`, `P_write-whole-path-create`,
  `P_mutation-contained`, `P_mutation-repair-validation`,
  `P_mutation-outcome-boundary`, `P_mutation-result-shape`,
  `P_mutation-diagnostics-bounded`, `P_mutation-precondition-error`, and
  `P_applied-invalid-result`, each of which describes an operation the server no
  longer serves. Reword `P_direct-does-not-load` and `P_first-index-call-loads`
  in terms of the three served tools. Retain every transport, authentication,
  lifecycle, search, graph, and safety promise unaffected by the removal. Every
  remaining `PENDING(issue-082)` and `PENDING(issue-083)` marker must go with the
  promise that carries it or be settled by removal, leaving no marker naming a
  declined issue. Retarget the frontmatter `description`, the introductory
  prose, and the `references` link from `adr-053` to `adr-055`, so the contract
  names the decision it implements rather than the one that decision superseded.
  Drop the `/crates/lib/superdev-core/src/sokf/mutation.rs#tools` include from
  the Definition: MCP accepts no mutation request, so those request types stop
  being this contract's to define.
- `contract-002-cli-superdev`: add the released
  `/crates/lib/superdev-core/src/sokf/mutation.rs#tools` include to the
  Definition, because `superdev sokf edit --request-json` and
  `sokf write --request-json` become the only public callers of `ExactEdit`,
  `EditRequest`, and `WriteRequest`, and `P_sokf-request-policy-fixed` and
  `P_sokf-mutation-json-shape` bind a shape no contract would otherwise define.
  Reword `P_sokf-retrieval-shares-service` so `sokf read` is bound to the shared
  service alone, while `sokf overview`, `search`, and `graph` keep their MCP
  correspondence. No command, argument, exit code, or stream changes.
- `contract-012-api-sokf-pi-file-tools`: move `lifecycle` to `deprecated`, and
  add a `references` link to `adr-055` beside the existing `adr-053` link. The
  contract recorded the routed adapter interface, and no live adapter remains.
  Its `tools` source region leaves `.pi/extensions/sokf.ts` with the
  registrations, so retarget `resource` and the Definition include at
  `/archive/pi/sokf-file-tools.ts`, which preserves those registrations
  verbatim. Remove every promise and criterion carrying `PENDING(issue-082)` or
  `PENDING(issue-083)`: each describes adapter behaviour that was never built
  and now never will be, and a deprecated contract must not defer to a declined
  issue. Retain the promises the delivered adapter kept — worktree discovery and
  precedence, cross-checkout refusal, source routing and its built criteria,
  read parity and its built criteria, the pinned-Pi evidence promise, and the
  stability promises — as the record of what the adapter did.
  That blanket removal applies only to sections the contract schema does not
  require. `Authentication`, `Errors`, and `Limits` are `required: true` for an
  `api` contract and hold nothing but PENDING promises today, so removing them
  wholesale would starve three required sections. Each promise in those three
  keeps its key, drops its PENDING marker, and is rewritten to record what the
  adapter actually did — not what the declined issues proposed. `P_local-authority`
  and `P_validation-diagnostic-limit` were built and are restated as delivered:
  mandatory agent-safe mutation authority with no model-facing override, and the
  200-line or 8-KiB extension-side diagnostic cap. `P_pi-errors` and
  `P_policy-errors` held only for routed `read` after issue-077 and never for
  routed `edit` or `write`, so each narrows to the read path and says so.
  `P_post-persistence-findings` was never built — mutation envelopes stayed
  model-visible file-tool content to the end — so it is restated as the unbuilt
  intent it was, attributed to the declined `issue-083`. No promise may be
  invented to fill a required section, and no unbuilt promise may be recorded as
  delivered.
- `contract-013-interface-sokf-pi-extension`: new internal interface contract with
  `resource: /.pi/extensions/sokf.ts`, matching the kind of the repository's
  other interface contracts. Its Definition materializes the three tool
  registrations and the `turn_end` handler from one marked source region. It must
  promise the unconditional turn-end `validate --fix`, that a valid run leaves
  the tree byte-identical and sends nothing, the freshness re-check against the
  working tree before sending, the first-report-triggers and later-report-does-not
  sequence with where that sequence resets, session-memory follow-up state keyed
  by canonical repository root with nothing persisted, the 200-line or 8-KiB
  message cap directing truncated output to `superdev validate`, the three
  registered tools and the absence of any `read`, `edit`, or `write`
  registration, and repository discovery and containment across a nested working
  directory and a linked worktree. It inherits the substance of
  `contract-012`'s `P_validation-follow-up`, `P_worktree-isolation`, and
  `P_validation-diagnostic-limit`, which retire with that contract; it must not
  reuse their keys, because a removed key is not reused.

## ADR decisions

- `adr-055-sokf-does-not-intercept-file-tools`: this plan implements it in full —
  the three-tool MCP surface, unrouted file tools, paths in graph results, and
  turn-end repair with fresh, bounded, session-scoped reporting. The ADR is
  already `active` and needs no change.
- `adr-053-sokf-file-tools-delegate-to-pi`: already `deprecated` and superseded
  by `adr-055`. BUILD must not edit it; a superseded ADR is history.

## Source and interface changes

`crates/lib/superdev-core/src/sokf/mcp.rs` must serve `sokf_search`,
`sokf_graph`, and `sokf_overview` and nothing else. The `sokf_resolve_source`,
`sokf_retrieve`, `sokf_edit`, and `sokf_write` tool functions must be removed
from the tool router, together with the request types that exist only to serve
them and the `SourceResolution` result type. `SokfService::retrieve()` and
`resolve_source()` must be removed with them. `SokfService::overview()`,
`search()`, `graph()`, `edit()`, and `write()` must remain: the last two serve
the command line and the workflow. `get_info()` must rewrite its instruction
string to name only the three served tools and to describe reaching a concept's
source through an ordinary file read of the path that search or graph returned.

`sokf_overview` must take an empty closed request object and return one text
content item. `sokf_graph` must render a repository-relative path beside each
named concept in `render_edges()` and `render_neighbours()`.

`crates/lib/superdev-core/src/sokf/mod.rs` must drop the re-exports of the
removed types and keep every other export unchanged.

`crates/lib/superdev-core/src/sokf/mutation.rs` must keep `edit()`, `write()`,
their inline repair, and `MutationResult` exactly as they are, because the
command line and the workflow depend on them. `GeneratedRegion` and
`generated_regions()` must be removed if nothing but the departed resolver used
them; BUILD must check for other callers before removing either.

`.pi/extensions/sokf.ts` must register `sokf_search`, `sokf_graph`, and
`sokf_overview` only. Every `read`, `edit`, and `write` registration must be
removed, along with `prepareReadPath()`, `routesRead()`, `routesMutation()`,
`sourceResolution()`, `restoreSpelling()`, `mutationDetails()`, `editDetails()`,
`displayDiff()`, `physicalTarget()`, `isWithin()`, `knowledgeRoot()`,
`isSokfAddress()`, the Pi file-tool factory imports, and the path-normalisation
adapter that existed to match Pi's argument preparation. `findRepository()` must
remain, serving MCP client keying and the working directory of the `superdev`
child process.

The `turn_end` handler must run `superdev validate --fix` rather than bare
`validate`, on every turn, with no precondition. The `knowledgeMutated` flag and
every assignment to it must be removed: the registrations that set it are going,
and a flag only the extension's own tools can set cannot observe a write through
`bash`. Replacing it with a cleverer signal would reintroduce the same class of
miss, so nothing replaces it. After the run, the
handler must re-read the reported paths and drop findings the tree no longer
carries, then send at most one message: triggering for the first report of a
pending sequence, visible and non-triggering afterwards. State must be a map
keyed by canonical repository root, held in session memory.

`.pi/extensions/sokf-mcp.ts` must drop the `SourceResolution` and
`GeneratedRegion` types and keep its process reuse, restart, abort, and shutdown
behaviour unchanged.

`crates/lib/superdev-core/tests/mcp_tools.rs` must drop every case for the
removed operations and gain cases for `sokf_overview` and for paths in graph
output. `scripts/test/fixtures/sokf-pi-paired.ts` must be deleted: it compares a
routed read against a built-in read, and no routed read remains.
`scripts/test/sokf-pi-adapter.test.mjs` and
`scripts/test/fixtures/sokf-pi-adapter-smoke.ts` must cover the three registered
tools and assert that the session registers no `read`, `edit`, or `write`.
`scripts/test/fixtures/sokf-mcp-fake.mjs` and
`scripts/test/sokf-mcp-client.test.mjs` must follow the changed tool set.
`crates/app/superdev/tests/cli.rs` must assert the new served roster.

The command-line interface, its arguments, its output, and its exit codes must
remain unchanged. `crates/lib/superdev-core/src/sokf/mutation.rs` must keep
`ExactEdit`, `EditRequest`, and `WriteRequest` inside a marked region, because
the `--request-json` forms of `superdev sokf edit` and `sokf write` remain their
public callers; the include of that region moves from `contract-003-api-sokf` to
`contract-002-cli-superdev`. `GeneratedRegion` leaves the region with the
resolver it served.

The removed registrations must be preserved verbatim as
`/archive/pi/sokf-file-tools.ts`, carrying the `tools` region
`contract-012-api-sokf-pi-file-tools` includes, as `/archive/claude-code/run.rs`
preserves `contract-009-interface-run-state`'s definition. No extension, build,
or test may load or reference the archived file.

`crates/lib/superdev-core/src/agent-instructions.md` must stop directing an agent
at `read path="sokf:<id>"` and must instead name a physical `knowledge/` path
reached from a `sokf_search` or `sokf_graph` result. `.agents/superdev.md` must
be regenerated from it by `cargo run -- sync`, never hand-edited, because the
sokf component owns and lock-claims that file.

`.pi/skills/sokf-authoring/SKILL.md` must stop describing the SOKF adapter,
routed `edit path="sokf:<id>"`, mutation-tool repair of generated blocks,
`applied: true`, and two automatic repair follow-ups. It must describe reading
and writing physical `knowledge/` paths with Pi's own tools, and one
repair-and-validate pass at turn end. `pack/pi/skills/sokf-authoring/SKILL.md`
must stay byte-identical to it.

`evals/sokf/behavioral.json` must name a physical `knowledge/` path wherever a
scenario names a `sokf:` path for `read`, `edit`, or `write`. Seven scenarios do:
`architecture-decision` and `direct-concept-reference` (`read`), `concept-edit`,
`knowledge-producing-code-change`, and `outward-facing-documentation` (`edit`),
`multi-step-knowledge-change` (a `read` whose path array carries both a `sokf:`
glob and a `knowledge/` glob, from which the `sokf:` alternative is dropped), and
`unknown-concept-id`. The first six keep their intent exactly. `unknown-concept-id`
is re-aimed: its prompt and `orderedCalls` must name a missing `knowledge/` path
rather than `sokf:command-router-policy`, and it must still require `sokf_search`
recovery and still forbid a broad filesystem scan, so it measures semantic
recovery from an unresolved concept instead of near-miss recovery on a routed
read. The roster, each sandbox, the acceptance threshold, and
`safetyCriticalScenarios` stay unchanged.
`scripts/test/sokf-behavior-fixtures.test.mjs` must follow it.

`contract-013-interface-sokf-pi-extension` must be authored in Block 2, when the
turn-end behaviour lands, so the mechanism is bound before Block 3 removes the
mutation-time repair it replaces. Its Definition materializes from a marked
region in `.pi/extensions/sokf.ts` spanning the three `pi.registerTool({...})`
calls and the `pi.on("turn_end", ...)` handler, and excludes the MCP transport
helpers, which `contract-003-api-sokf` already governs.

## Knowledge changes

BUILD must update `architecture`, `software-components`, `testing-strategy`,
`security-requirements`, `development-commands`, and the `contract-003-api-sokf`
entry in `/knowledge/contracts/index.md` for the three-tool surface, the absence
of routed file tools, paths in graph results, and turn-end repair as the single
point of consistency. No document may retain the routed read, edit, or write
behaviour, a served-tool count or list that predates this change, a claim that
knowledge mutation passes through an agent-safe MCP path, a claim that a
`sokf:<id>` address is how an agent reads a concept, or a reference to the
deleted paired read harness. `development-commands` and `testing-strategy` must
describe the changed script suite.

No document may describe the standing instruction or the `sokf-authoring` skill
as directing an agent at a `sokf:` address, and `development-commands` must
describe the skill's changed guidance.

Documents must continue to describe `superdev sokf edit`, `superdev sokf write`,
and their inline repair accurately, because those are unchanged. Keep the issue
and this plan current through BUILD evidence and acceptance. Let validation
regenerate contract, issue, plan, and ADR indexes and all source include and link
blocks.

## Documentation changes

The documentation map triggers the following surfaces.

- `readme`: update `/README.md` because the MCP tool names, the tool count, and
  how an agent reaches canonical knowledge are user-visible. Verify with
  `npm run check:docs`.
- `contributor-guide`: update `/CONTRIBUTING.md` and
  `/knowledge/development-commands.md` for the removed paired read harness and
  the changed script suite. Verify with `npm run check:docs`.
- `canonical-knowledge`: update the contracts, architecture, components, tests,
  and security requirements listed above. Generate with
  `cargo run -- validate --fix` and verify with `npm run check:validate`.
- `changelog`: add the removal under `/CHANGELOG.md` `[Unreleased]`, naming the
  replaced MCP tools as a breaking change to an unreleased surface. Verify with
  `npm run check:docs`.
- `cli-reference`: triggered because `contract-002-cli-superdev`'s materialized
  Definition gains the mutation request region `contract-003-api-sokf` releases.
  Command, argument, help, exit-code, stream, and workflow protocol shapes stay
  unchanged. Generate with `cargo run -- validate --fix` and verify with
  `cargo test -p superdev --test cli` and `npm run check:validate`.
- `documentation-site`: not triggered because the repository has no site source
  or renderer. Confirm with `npm run check:docs`.
- Packaging copies: triggered because `.pi/skills/sokf-authoring/SKILL.md`
  changes in Block 4 and its `pack/pi/skills/` mirror must stay byte-identical,
  and because `.agents/superdev.md` must be regenerated from
  `crates/lib/superdev-core/src/agent-instructions.md`. Generate with
  `cargo run -- sync` and verify with `npm run check:blueprint`.
- Final verification: `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo nextest run --workspace && cargo test --doc --workspace && RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps && npm run test:launcher && npm run test:scripts && npm run check:docs && npm run check:validate && npm run check:blueprint && npm run coverage:check`.

## Work blocks

### Block 1: Serve an overview tool and put paths in graph results

- [ ] Done.
- Dependencies: none.
- Areas: `crates/lib/superdev-core/src/sokf/mcp.rs`, `crates/lib/superdev-core/tests/mcp_tools.rs`, and `contract-003-api-sokf`.
- Outcome: MCP serves `sokf_overview` alongside its existing tools, and every concept a graph result names carries its repository-relative path. Nothing is removed, so the extension keeps working throughout.
- Verification: `cargo test -p superdev-core sokf && cargo test -p superdev-core --test mcp_tools`.
- Tests: cases cover `contract-003-api-sokf AC_overview-content`, `AC_overview-no-arguments`, `AC_graph-carries-paths`, and the retargeted `P_overview-warning-cap`. One case asserts the `sokf_overview` request schema is an empty object with `additionalProperties: false` and that a call with an unknown property is refused. One case asserts the overview text carries the knowledge name, a concept count, a tree entry, and index state. Graph cases assert a path beside each concept for the whole edge map and for one concept's neighbours, and that the path resolves to an existing file in the fixture.
- Structural evidence: the tool list contains `sokf_overview`; the overview text equals `SokfService::overview()` output rather than a second rendering; graph paths are repository-relative with forward slashes on every platform, asserted by comparing against a path built from `Path` components rather than by substring.
- Documentation: implement and materialize the scoped `contract-003-api-sokf` declarations, then run `cargo run -- validate --fix` followed by `npm run check:validate`.

### Block 2: Turn end repairs, re-checks, and reports

- [ ] Done.
- Dependencies: none.
- Areas: `.pi/extensions/sokf.ts`, `contract-013-interface-sokf-pi-extension`, `scripts/test/sokf-pi-adapter.test.mjs`, and `scripts/test/fixtures/sokf-pi-adapter-smoke.ts`.
- Outcome: every turn ends with one `validate --fix`, a freshness re-check, and at most one bounded message, and `contract-013-interface-sokf-pi-extension` binds that behaviour. This lands before Block 3 removes mutation-time repair, so no intermediate state leaves the knowledge unrepaired and no interval leaves the turn-end mechanism uncontracted.
- Verification: `node --test scripts/test/sokf-pi-adapter.test.mjs`.
- Tests: cases prove that the run happens on a turn that changed nothing, on a turn that changed knowledge through a file tool, and on a turn that changed knowledge by writing directly to the fixture tree; that `validate --fix` is the command invoked; that a finding resolved between the run and the send is not reported; that a report surviving no finding sends no message; that the first report of a sequence triggers a turn and a later one does not; that state is keyed by repository root and two roots do not share it; and that a message over 200 lines or 8 KiB is truncated and names `superdev validate`.
- Structural evidence: no `knowledgeMutated` flag or equivalent precondition remains in the handler, and a test proves the run happens after a turn in which none of the extension's own tools were called. A turn that touched no knowledge leaves the fixture tree byte-identical, so the unconditional run performs no spurious write. The freshness re-check reads the reported paths again after validation rather than filtering a captured report. Follow-up state is held in a session-scoped map and no file is written, asserted by comparing the fixture tree before and after a reporting turn.
- Documentation: author `contract-013-interface-sokf-pi-extension`, add the marked source region it includes, then run `cargo run -- validate --fix` followed by `npm run check:validate`. The remaining surfaces change with Block 3.

### Block 3: Stop intercepting the file tools and remove the routed operations

- [ ] Done.
- Dependencies: Block 1, Block 2.
- Areas: `.pi/extensions/sokf.ts`, `.pi/extensions/sokf-mcp.ts`, `/archive/pi/sokf-file-tools.ts`, `crates/lib/superdev-core/src/sokf/mcp.rs`, `crates/lib/superdev-core/src/sokf/mod.rs`, `crates/lib/superdev-core/src/sokf/mutation.rs`, `crates/lib/superdev-core/tests/mcp_tools.rs`, `crates/app/superdev/tests/cli.rs`, `scripts/test/fixtures/sokf-pi-paired.ts`, `scripts/test/fixtures/sokf-pi-adapter-smoke.ts`, `scripts/test/fixtures/sokf-mcp-fake.mjs`, `scripts/test/sokf-pi-adapter.test.mjs`, `scripts/test/sokf-mcp-client.test.mjs`, `contract-002-cli-superdev`, `contract-003-api-sokf`, `contract-012-api-sokf-pi-file-tools`, `/README.md`, `/CONTRIBUTING.md`, `/CHANGELOG.md`, `/knowledge/contracts/index.md`, `architecture`, `software-components`, `testing-strategy`, `security-requirements`, and `development-commands`.
- Outcome: a Pi session gets Pi's own file tools and exactly three SOKF tools, MCP serves nothing else, and the documentation describes the implemented behaviour.
- Verification: `cargo test -p superdev-core sokf && cargo test -p superdev-core --test mcp_tools && cargo test -p superdev --test cli && node --test scripts/test/sokf-pi-adapter.test.mjs && node --test scripts/test/sokf-mcp-client.test.mjs && npm run check:docs && npm run check:validate && npm run check:blueprint`.
- Tests: MCP cases assert the served roster is exactly `sokf_graph`, `sokf_overview`, and `sokf_search`, and that a call naming a removed tool fails as unknown. One case asserts the `get_info()` instruction string names those three and contains none of `sokf_read`, `sokf_resolve_source`, `sokf_retrieve`, `sokf_edit`, or `sokf_write`, reading the value the method returns rather than a copy. Adapter cases assert the session registers exactly those three tools and registers no `read`, `edit`, or `write`. A case proves a `knowledge/` path read through Pi's built-in returns the file's exact bytes including frontmatter, and that an excerpt copied from it works unchanged as an `edit` anchor. Command-line cases prove `superdev sokf edit` and `superdev sokf write` retain their applied, validation, resolved, final, and diff output.
- Structural evidence: no SOKF source imports a Pi file-tool factory; `rg` over `.pi/extensions/` finds no `createReadTool`, `createEditTool`, `createWriteTool`, `createReadToolDefinition`, or `withFileMutationQueue`. `scripts/test/fixtures/sokf-pi-paired.ts` is deleted rather than disabled. A workflow-transition test still passes unchanged, proving `SokfService::edit` kept its inline repair and refiling. `cargo run -- validate` runs twice with a clean second pass, proves `contract-012` is `deprecated` and materializes its Definition from `/archive/pi/sokf-file-tools.ts`, proves `contract-002-cli-superdev` materializes the released mutation request region, and proves no `PENDING(issue-082)` or `PENDING(issue-083)` marker remains anywhere in canonical knowledge.
- Documentation: update `/README.md`, `/CONTRIBUTING.md`, `/CHANGELOG.md`, and the canonical knowledge listed above, run `cargo run -- validate --fix` twice, then run `npm run check:docs`, `npm run check:validate`, and `npm run check:blueprint`.

### Block 4: Retire the routed-authoring guidance

- [ ] Done.
- Dependencies: Block 3.
- Areas: `crates/lib/superdev-core/src/agent-instructions.md`, `/.agents/superdev.md`, `.pi/skills/sokf-authoring/SKILL.md`, `pack/pi/skills/sokf-authoring/SKILL.md`, `evals/sokf/behavioral.json`, `scripts/test/sokf-behavior-fixtures.test.mjs`, and `development-commands`.
- Outcome: the standing instruction, the authoring skill, and the behaviour evaluations direct an agent at a physical `knowledge/` path and at one turn-end repair, so no guidance survives that names an operation the session no longer serves. It follows Block 3 because guidance describes delivered behaviour; between the two blocks the guidance is stale but nothing is broken, and no release sits between them.
- Verification: `cargo run -- sync && npm run check:blueprint && node --test scripts/test/sokf-behavior-fixtures.test.mjs && npm run check:validate`.
- Tests: the fixture test asserts the scenario roster, the sandboxes, the acceptance threshold, and `safetyCriticalScenarios` are unchanged, and that no scenario names a `sokf:` path for `read`, `edit`, or `write` — an assertion all seven converted scenarios must satisfy, not three. One case asserts `direct-concept-reference` names a physical `knowledge/` path and still forbids `sokf_search`, so the scenario keeps measuring direct addressing rather than a broad scan. One case asserts `concept-edit` edits a physical `knowledge/` path and still requires its schema read first. One case asserts `multi-step-knowledge-change` keeps its `knowledge/schemas/*.md` read alternative and no longer offers a `sokf:` one. One case asserts the re-aimed `unknown-concept-id` names a missing `knowledge/` path, still requires `sokf_search` after the failed read, and still forbids a broad filesystem scan, so it measures semantic recovery rather than the near-miss behaviour the routed read used to provide.
- Structural evidence: `rg` over `/.agents/`, `.pi/skills/`, `pack/pi/skills/`, and `evals/` finds no `read path="sokf:`, no `edit path="sokf:`, and no promise of two automatic repair follow-ups. `.agents/superdev.md` differs from its previous revision only where `cargo run -- sync` regenerated it from the edited source, and `npm run check:blueprint` exits 0. `pack/pi/skills/sokf-authoring/SKILL.md` is byte-identical to `.pi/skills/sokf-authoring/SKILL.md`, compared as bytes rather than by rendering.
- Documentation: update `development-commands` for the skill's changed guidance, run `cargo run -- validate --fix`, then run `npm run check:validate` and `npm run check:blueprint`.

## Build state

Current block: 1. Attempts: 0. Final corrections: 0. Blocker: none.

## Implementation decisions

none. BUILD records only local choices that do not change the approved contracts
or ADR.

## Follow-up issues

none. Registering a Pi-side tool for rendered concepts or section addressing, if
the removal proves to have cost something, requires a separate issue. Semantic
ranking, index lifecycle, MCP transport, and search behaviour are unaffected and
require separate issues.

## Completion evidence

Scope review and approval are pending; BUILD evidence is pending.

Workflow default branch: main.

<!-- sokf:links -->
[sokf:adr-055-sokf-does-not-intercept-file-tools]: /knowledge/adrs/active/adr-055-sokf-does-not-intercept-file-tools.md
[sokf:issue-077-sokf-file-tool-parity]: /knowledge/issues/done/issue-077-sokf-file-tool-parity.md
[sokf:issue-095-sokf-stops-intercepting-file-tools]: /knowledge/issues/open/issue-095-sokf-stops-intercepting-file-tools.md
