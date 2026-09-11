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
  concept count, tree, and index state — the output `SokfService::overview()`
  already produces. It must carry no rendered concept, no section addressing, and
  no line window.
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
- The unconditional run must leave the tree byte-identical when the knowledge is
  already valid. Against this repository's 279 concepts it costs about 1.3
  seconds in a debug build and reports `repaired: 0 file(s)`, so a turn that
  touched no knowledge pays a bounded cost and changes nothing.
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
  adapter interface it defines ceases to exist.
- The routed promises on `contract-003-api-sokf` must be removed rather than left
  describing absent operations, and its retained promises must be accurate
  against the three served tools.
- No pack copy or `.superdev/lock.toml` claim exists for `.pi/extensions/sokf.ts`,
  so `sync` will not overwrite it; the `pack/` mirror of
  `.pi/skills/sokf-authoring/SKILL.md` must stay byte-identical to its source if
  that skill changes.

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
  declined issue.
- `contract-012-api-sokf-pi-file-tools`: move `lifecycle` to `deprecated`. The
  contract defined the routed adapter interface, and no adapter remains to
  define. Its `tools` source region in `.pi/extensions/sokf.ts` disappears with
  the registrations, so BUILD must remove the Definition include rather than
  leave it naming an absent region.

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

The command-line interface, its request and output types, and the generated CLI
reference must remain unchanged.

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
- `cli-reference`: not triggered because command, argument, help, exit-code,
  stream, and workflow protocol shapes are unchanged. Confirm with
  `npm run check:docs`.
- `documentation-site`: not triggered because the repository has no site source
  or renderer. Confirm with `npm run check:docs`.
- Packaging copies: not triggered unless BUILD changes
  `.pi/skills/sokf-authoring/SKILL.md`, whose `pack/pi/skills/` mirror must then
  stay byte-identical. Confirm with `npm run check:blueprint`.
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
- Areas: `.pi/extensions/sokf.ts`, `scripts/test/sokf-pi-adapter.test.mjs`, and `scripts/test/fixtures/sokf-pi-adapter-smoke.ts`.
- Outcome: every turn ends with one `validate --fix`, a freshness re-check, and at most one bounded message. This lands before Block 3 removes mutation-time repair, so no intermediate state leaves the knowledge unrepaired.
- Verification: `node --test scripts/test/sokf-pi-adapter.test.mjs`.
- Tests: cases prove that the run happens on a turn that changed nothing, on a turn that changed knowledge through a registered tool, and on a turn that changed knowledge by writing directly to the fixture tree; that `validate --fix` is the command invoked; that a finding resolved between the run and the send is not reported; that a report surviving no finding sends no message; that the first report of a sequence triggers a turn and a later one does not; that state is keyed by repository root and two roots do not share it; and that a message over 200 lines or 8 KiB is truncated and names `superdev validate`.
- Structural evidence: no `knowledgeMutated` flag or equivalent precondition remains in the handler, and a test proves the run happens after a turn in which none of the extension's own tools were called. A turn that touched no knowledge leaves the fixture tree byte-identical, so the unconditional run performs no spurious write. The freshness re-check reads the reported paths again after validation rather than filtering a captured report. Follow-up state is held in a session-scoped map and no file is written, asserted by comparing the fixture tree before and after a reporting turn.
- Documentation: none in this block; the surfaces change with Block 3.

### Block 3: Stop intercepting the file tools and remove the routed operations

- [ ] Done.
- Dependencies: Block 1, Block 2.
- Areas: `.pi/extensions/sokf.ts`, `.pi/extensions/sokf-mcp.ts`, `crates/lib/superdev-core/src/sokf/mcp.rs`, `crates/lib/superdev-core/src/sokf/mod.rs`, `crates/lib/superdev-core/src/sokf/mutation.rs`, `crates/lib/superdev-core/tests/mcp_tools.rs`, `crates/app/superdev/tests/cli.rs`, `scripts/test/fixtures/sokf-pi-paired.ts`, `scripts/test/fixtures/sokf-pi-adapter-smoke.ts`, `scripts/test/fixtures/sokf-mcp-fake.mjs`, `scripts/test/sokf-pi-adapter.test.mjs`, `scripts/test/sokf-mcp-client.test.mjs`, `contract-003-api-sokf`, `contract-012-api-sokf-pi-file-tools`, `/README.md`, `/CONTRIBUTING.md`, `/CHANGELOG.md`, `/knowledge/contracts/index.md`, `architecture`, `software-components`, `testing-strategy`, `security-requirements`, and `development-commands`.
- Outcome: a Pi session gets Pi's own file tools and exactly three SOKF tools, MCP serves nothing else, and the documentation describes the implemented behaviour.
- Verification: `cargo test -p superdev-core sokf && cargo test -p superdev-core --test mcp_tools && cargo test -p superdev --test cli && node --test scripts/test/sokf-pi-adapter.test.mjs && node --test scripts/test/sokf-mcp-client.test.mjs && npm run check:docs && npm run check:validate && npm run check:blueprint`.
- Tests: MCP cases assert the served roster is exactly `sokf_graph`, `sokf_overview`, and `sokf_search`, and that a call naming a removed tool fails as unknown. One case asserts the `get_info()` instruction string names those three and contains none of `sokf_read`, `sokf_resolve_source`, `sokf_retrieve`, `sokf_edit`, or `sokf_write`, reading the value the method returns rather than a copy. Adapter cases assert the session registers exactly those three tools and registers no `read`, `edit`, or `write`. A case proves a `knowledge/` path read through Pi's built-in returns the file's exact bytes including frontmatter, and that an excerpt copied from it works unchanged as an `edit` anchor. Command-line cases prove `superdev sokf edit` and `superdev sokf write` retain their applied, validation, resolved, final, and diff output.
- Structural evidence: no SOKF source imports a Pi file-tool factory; `rg` over `.pi/extensions/` finds no `createReadTool`, `createEditTool`, `createWriteTool`, `createReadToolDefinition`, or `withFileMutationQueue`. `scripts/test/fixtures/sokf-pi-paired.ts` is deleted rather than disabled. A workflow-transition test still passes unchanged, proving `SokfService::edit` kept its inline repair and refiling. `cargo run -- validate` runs twice with a clean second pass, proves `contract-012` is `deprecated` and carries no include naming an absent region, and proves no `PENDING(issue-082)` or `PENDING(issue-083)` marker remains anywhere in canonical knowledge.
- Documentation: update `/README.md`, `/CONTRIBUTING.md`, `/CHANGELOG.md`, and the canonical knowledge listed above, run `cargo run -- validate --fix` twice, then run `npm run check:docs`, `npm run check:validate`, and `npm run check:blueprint`.

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
