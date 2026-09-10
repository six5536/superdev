---
type: Plan
id: plan-077-sokf-file-tool-parity
title: SOKF-routed reads match Pi's built-in read contract
description: Split the mixed MCP read operation into source resolution and semantic retrieval, then route Pi's built-in read through the resolved canonical target.
lifecycle: open
phase: scope
branch: work/077-sokf-file-tool-parity
links:
  - rel: implements
    to: issue-077-sokf-file-tool-parity
---

# Plan: SOKF-routed reads match Pi's built-in read contract

## Goal and boundaries

Implement [SOKF-routed reads match Pi's built-in read contract][sokf:issue-077-sokf-file-tool-parity].
A physical knowledge path and its `sokf:<id>` alias must expose the same Pi 0.85.1
read behavior in a normal checkout or linked Git worktree. SOKF adds identity
routing and repository containment only.

The installed Pi factories, operations interfaces, result types, renderers,
`createReadTool()`, `ReadToolDetails`, and `resolveReadPathAsync()` are the
executable references. The adapter must delegate to those interfaces or use a
narrow adapter proven byte-for-byte against them. The SOKF Model Context Protocol
(MCP) contract is unreleased, so this plan deliberately replaces `sokf_read`
instead of preserving it.

This plan is the first of three that share `contract-003-api-sokf`,
`contract-012-api-sokf-pi-file-tools`, and
`adr-053-sokf-file-tools-delegate-to-pi`. Deferral markers on those contracts name
the issue that settles each promise. This plan removes only `PENDING(issue-077)`
markers and leaves every `PENDING(issue-082)` and `PENDING(issue-083)` marker in
place. It leaves `adr-053` in `draft` because
[issue-083][sokf:issue-083-sokf-validation-follow-ups] settles its last decisions.

The work excludes routed edit and write behavior, the mutation outcome boundary,
validation follow-up delivery, file-tool prompt metadata, a semantic result
dialect in Pi's `read` slot, approximate copies of Pi algorithms, weakened SOKF
policy, and the general validator symlink walk tracked by issue-031.

## Requirements

The implementation must satisfy the following settled requirements.

- `read path="sokf:<id>"` must resolve the identity to its canonical
  repository-relative path and return the exact UTF-8 source bytes through Pi's
  built-in read behavior.
- Routed reads must preserve 1-indexed `offset` and `limit`, 2,000-line and 50 KB
  head truncation, continuation messages, `ReadToolDetails.truncation`, access
  errors, out-of-range errors, cancellation, metadata, and rendering.
- Pi's file `read` must reject `sokf:` overview and section-qualified addresses
  with concise guidance naming `sokf_search`, the semantic tool the session
  already registers.
- The routed `read` description and prompt guidance must stop advertising the
  refused overview and section address forms. That correction states implemented
  behavior and must not settle `contract-012-api-sokf-pi-file-tools
  P_prompt-ownership` or `P_file-prompt-rules`, which remain
  `PENDING(issue-083)`.
- The adapter must discover the canonical active checkout root from either a
  `.git` directory or a linked-worktree `.git` pointer file, from the checkout
  root or any descendant. When the upward walk finds no `.git` marker at all, it
  must fall back to the nearest ancestor carrying `.superdev/config.toml`, so a
  managed non-Git repository keeps the SOKF routing it has today. That ordering
  is exhaustive and fixed: any `.git` marker on the walk wins over the
  configuration marker. The selected root must own source containment, MCP
  process reuse, and index activity. Git's shared common directory and main
  checkout must never replace it. A routed path entering another checkout must
  fail before access.
- The MCP server's initialization instructions must name only the tools the
  server serves. They must stop naming `sokf_read` and stop directing a file
  read at the `sokf:` overview address.
- MCP must remove the mixed-purpose `sokf_read` tool. A new `sokf_resolve_source`
  tool must accept an unqualified `sokf:<id>` or a physical knowledge path and
  return the repository-relative `knowledge/` ingress path, the canonical
  repository-relative target, existence, and generated-authority metadata without
  semantic content.
- The Pi adapter must apply a narrow path-preparation adapter before calling the
  resolver. Paired tests must prove its output byte-for-byte against the path
  values observed at Pi 0.85.1 factory operation boundaries, and the adapter must
  not deep-import unexported package internals. Read routing must preserve
  `resolveReadPathAsync()` handling for one leading `@`, `~`, Unicode spaces,
  macOS AM/PM spacing, Normalization Form D (NFD), curly quotes, and combined
  variants.
- An existing regular file must return `exists: true`. A missing physical
  destination must canonicalize its nearest existing ancestor, append the
  normalized missing suffix, retain a logical ingress under `knowledge/`, keep its
  canonical target inside the repository, and return `exists: false`.
- An existing or ancestor symlink that resolves to a file or directory inside the
  repository must use the contained target, including a target outside
  `knowledge/`. A symlink that resolves outside the repository, including one
  followed before a missing suffix, must fail containment before any read.
- Direct physical arguments outside `knowledge/`, missing identities, overview
  addresses, section-qualified addresses, and directories must remain errors.
  Generated projections must resolve for reads while naming the authoritative
  source.
- MCP must expose semantic overview, rendered-concept, and section retrieval as
  `sokf_retrieve`. `sokf_retrieve` must retain the current semantic output and
  line-window behavior for those address forms, refuse physical file paths, and
  require no `sokf_read` compatibility alias or deprecation period.
- The routed MCP objects must use exact closed schemas with no additional
  properties. `sokf_resolve_source` accepts `{ path: string }` and returns
  structured content `{ ingressPath: string, canonicalPath: string, exists:
  boolean, generatedRegions: GeneratedRegion[] }` together with exactly one
  `{ type: "text", text: string }` content item carrying the pretty-printed JSON
  of that same structured content and nothing else, matching the existing
  `mutation_result()` convention. The adapter reads the structured half and
  ignores the text item, as it already does for `sokf_search` and `sokf_graph`,
  so the mirrored JSON adds no semantic content. `GeneratedRegion` is
  `{ startLine: integer >= 1, endLine: integer >= startLine, authoritativePath:
  string, authoritativeRegion?: string }`. `sokf_retrieve` accepts `{ path:
  string, offset?: integer >= 1, limit?: integer >= 1 }` and returns one text
  content item without structured content.
- The adapter must join the resolver's canonical repository-relative target to the
  repository root and pass that absolute canonical execution target to Pi's read
  factory. A narrow presentation wrapper must restore the caller's path spelling
  in Pi's model-visible read notices without changing other result bytes.
- A copied routed excerpt, including frontmatter, must be byte-identical to the
  corresponding source range, so it works unchanged as an exact-replacement
  anchor.
- A paired harness must compare built-in and routed content, details, thrown
  errors, and renderer inputs exactly. The repository already pins
  `@earendil-works/pi-coding-agent` 0.85.1 as a test dependency; BUILD must keep
  that exact pin, load the pinned package rather than a `pi` executable on
  `PATH`, and fail rather than skip parity tests when it cannot load. The
  harness may normalize only unavoidable canonical-physical-versus-virtual
  target spelling. It must execute from both the repository root and a nested
  working directory.
- Routed edit and write must keep their current behavior until
  [issue-082][sokf:issue-082-sokf-mutation-parity] changes it. This plan must not
  regress the existing mutation path while replacing the read path.
- Existing shipped and materialized copies must remain synchronized. Repository
  inspection found only `.pi/skills/sokf-authoring/SKILL.md` mirrored under
  `pack/pi/skills/`; the project-local SOKF adapter has no pack copy or
  `.superdev/lock.toml` claim.

## Contract changes

- `contract-003-api-sokf`: settle every `PENDING(issue-077)` promise, including
  `AC_instructions-name-served-tools`, which the contract already declares.
  Replace the
  mixed-purpose `sokf_read` definition with materialized `sokf_resolve_source` and
  `sokf_retrieve` definitions. Bind the resolver and retrieval requests and
  results under `P_routed-schemas`, `AC_resolve-schema`, and `AC_retrieve-schema`;
  both objects reject additional properties. Settle `P_active-worktree-root`,
  `AC_worktree-other-checkout-refused`, `P_source-resolution`,
  `AC_source-identity`, `AC_source-contained`, `AC_source-section-refused`,
  `P_semantic-retrieve`, `AC_semantic-addresses`, `AC_semantic-physical-refused`,
  `P_no-read-alias`, `AC_instructions-name-served-tools`,
  `P_direct-does-not-load`, `P_first-index-call-loads`,
  `P_direct-retrieval-skips-index`, `P_parse-error-quoted`, and
  `P_overview-warning-cap`. Retain every `PENDING(issue-082)` and
  `PENDING(issue-083)` marker unchanged, including `P_routed-schemas` itself,
  which stays pending until its mutation criteria settle. Retain all unaffected
  search, graph, transport, authentication, lifecycle, and safety promises.
- `contract-012-api-sokf-pi-file-tools`: settle every `PENDING(issue-077)`
  promise: `AC_worktree-discovery`, `AC_worktree-cross-checkout-refused`,
  `P_source-routing`, `AC_source-existing`, `AC_source-missing`,
  `AC_source-nested-cwd`, `AC_source-pi-preparation`,
  `AC_source-contained-symlink`, `AC_source-escaping-symlink`, `P_read-parity`,
  `AC_read-exact-source`, `AC_read-semantic-refused`,
  `AC_read-description-accurate`, `AC_worktree-git-precedence`, and
  `P_pinned-pi`. BUILD must replace its whole-source include with one marked
  source region named `tools` in `.pi/extensions/sokf.ts` and materialize the
  changed declaration. That region must span the five `pi.registerTool({...})`
  calls for `read`, `edit`, `write`, `sokf_search`, and `sokf_graph`, so the
  Definition still declares every tool interface this contract's promises
  govern, and must exclude the transport helpers, diff rendering, and the
  `turn_end` validation handler that issue-082 and issue-083 own. Retain every
  marker naming issue-082 or
  issue-083; BUILD may correct the routed `read` description text that advertises
  refused address forms without settling `P_prompt-ownership` or
  `P_file-prompt-rules`.

## ADR decisions

- `adr-053-sokf-file-tools-delegate-to-pi`: this plan implements its identity
  resolution, Pi delegation, narrow byte-proven adapter, active-checkout
  containment, nearest-existing-ancestor resolution, contained-versus-escaping
  symlink, and unreleased-MCP-break decisions. The ADR remains `draft` because
  [issue-083][sokf:issue-083-sokf-validation-follow-ups] settles its last
  decisions. BUILD must not stabilize it in this plan and must not remove or
  reword the mutation or validation-state decisions that the later issues own.

## Source and interface changes

`crates/lib/superdev-core/src/sokf/mcp.rs` must replace `sokf_read` with
`sokf_resolve_source` and `sokf_retrieve`. MCP must declare the exact closed
request and result schemas from Requirements and `contract-003-api-sokf
P_routed-schemas` in materialized source regions. Resolution must remain a narrow
machine operation for unqualified identities and contained physical paths.
A successful resolver result must pair its structured content with exactly one
text item holding that content's pretty-printed JSON, through the same helper
shape `mutation_result()` already uses. Retrieval must retain existing overview,
rendered-concept, section, line-window, index-loading, and parse-error behavior
while refusing physical paths. No compatibility alias remains. `get_info()` must
rewrite its instruction string so it names only served tools and stops naming
`sokf_read` or a file read of the `sokf:` overview address; that string lies
outside the marked `tools` region, so BUILD must change it explicitly. The MCP
server must treat the canonical active checkout root as its repository,
including when `.git` is a linked-worktree pointer file, and must refuse a path
entering another checkout.

`crates/lib/superdev-core/src/sokf/mutation.rs` must expose each generated
region's inclusive line bounds, authoritative repository-relative path, and
optional source-region name so the resolver can report generated authority. Its
mutation behavior is otherwise unchanged in this plan.

`crates/lib/superdev-core/src/sokf/mod.rs` must re-export every new public
resolver type through the crate's flat SOKF surface, as
[coding-standards][sokf:coding-standards] requires, and must keep its existing
exports unchanged.

`.pi/extensions/sokf.ts` must discover the canonical active checkout root from
either a `.git` directory or a linked-worktree pointer file, including from a
nested working directory, and must key MCP clients by that root. `findRepository()`
must keep its existing `.superdev/config.toml` acceptance as a ranked fallback
rather than dropping it: the walk selects the nearest ancestor carrying a `.git`
marker, and only when no such ancestor exists does it select the nearest
ancestor carrying `.superdev/config.toml`. Today's single-pass either-marker
walk does not express that precedence and must change accordingly. It must resolve
before delegation and spread the complete Pi read tool definition so the schema,
argument preparation, metadata, result construction, and renderer remain
Pi-owned. For a physical argument, the adapter must apply a narrow
path-preparation adapter that matches the path values observed at Pi 0.85.1
factory operation boundaries. The adapter must join the resolver's canonical
repository-relative target to the repository root and pass that absolute canonical
execution target to the read factory. A narrow presentation wrapper must restore
the caller's path spelling in Pi's model-visible read notices without changing
other result bytes. The routed `read` registration must describe only the
address forms it accepts and must point a refused overview or section address at
`sokf_search`. The existing `edit` and `write` registrations keep their current
MCP calls; only their repository discovery changes with the shared root helper.

`.pi/extensions/sokf-mcp.ts` must adopt the renamed retrieval and typed
source-resolution transports. Its process reuse, restart, abort, and shutdown
contracts remain unchanged.

The contract Definition blocks must materialize the marked Rust and TypeScript
source declarations through `cargo run -- validate --fix`. BUILD must add the
`sokf:begin tools` and `sokf:end tools` markers to `.pi/extensions/sokf.ts`
around the five tool registrations named under Contract changes. `package.json` and
`package-lock.json` already carry the exact `@earendil-works/pi-coding-agent`
0.85.1 test-dependency pin, so BUILD must preserve it rather than add one and
must make the paired harness import that pinned package. The command-line
interface, its request and output types, and the generated CLI reference must
remain unchanged.

Primary implementation and evidence files are `.pi/extensions/sokf.ts`,
`.pi/extensions/sokf-mcp.ts`, `crates/lib/superdev-core/src/sokf/mcp.rs`,
`crates/lib/superdev-core/src/sokf/mutation.rs`,
`crates/lib/superdev-core/src/sokf/mod.rs`,
`crates/lib/superdev-core/tests/mcp_tools.rs`,
`scripts/test/fixtures/sokf-mcp-fake.mjs`,
`scripts/test/fixtures/sokf-pi-adapter-smoke.ts`,
`scripts/test/sokf-pi-adapter.test.mjs`, and
`scripts/test/sokf-mcp-client.test.mjs`.

## Knowledge changes

BUILD must narrow and materialize the adapter contract Definition, settle every
`PENDING(issue-077)` promise on both contracts, and update `architecture`,
`software-components`, `testing-strategy`, `security-requirements`,
`development-commands`, and the `contract-003-api-sokf` entry in
`/knowledge/contracts/index.md` for the `sokf_resolve_source` and
`sokf_retrieve` split, the removal of `sokf_read`, the resulting served-tool
names and count, exact source reads, active-checkout containment, the ranked
`.git`-then-`.superdev/config.toml` discovery ordering, and the resolver's
nearest-existing-ancestor and contained-symlink rules. No document may retain
the old rendered-virtual-read behavior, the mixed `sokf_read` operation, a
served-tool count or list that predates the split, a
canonical-knowledge-root-only target rule, a claim that discovery accepts either
marker without precedence, or the claim that paired adapter evidence runs only
when `pi` is on `PATH`. Adding the absent `contract-011-interface-workflow` and
`contract-012-api-sokf-pi-file-tools` entries to that contracts index is out of
scope; only the stale `contract-003-api-sokf` entry changes. Documents must
continue to describe mutation and validation behavior as the later issues define
it. Keep the issue and this plan current through BUILD evidence and acceptance.
Let validation regenerate contract, issue, and plan indexes and all source
include and link blocks.

## Documentation changes

The documentation map triggers the following surfaces.

- `readme`: update `/README.md` because the MCP tool names, tool count, retrieval
  guidance, and routed read behavior are user-visible. Verify with
  `npm run check:docs`.
- `contributor-guide`: update `/CONTRIBUTING.md` and
  `/knowledge/development-commands.md` to name the paired adapter harness and the
  pinned Pi 0.85.1 test dependency within `npm run test:scripts`, and to drop the
  claim that the harness runs only when `pi` is on `PATH`. Verify with
  `npm run check:docs`.
- `canonical-knowledge`: update the contracts, architecture, components, tests,
  and security requirements listed above. Generate with
  `cargo run -- validate --fix` and verify with `npm run check:validate`.
- `changelog`: add the routed-read change under `/CHANGELOG.md` `[Unreleased]`.
  Verify with `npm run check:docs`.
- `cli-reference`: not triggered because command, argument, help, exit-code,
  stream, and workflow protocol shapes remain unchanged.
- `documentation-site`: not triggered because the repository still has no site
  source or renderer. Confirm with `npm run check:docs`.
- Packaging copies: not triggered because this plan changes no packaged item. The
  authoring skill and its `pack/` mirror change with
  [issue-083][sokf:issue-083-sokf-validation-follow-ups]. Confirm with
  `npm run check:blueprint`.
- Final verification: `cargo fmt --all -- --check && cargo clippy --workspace --all-targets -- -D warnings && cargo nextest run --workspace && cargo test --doc --workspace && RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps && npm run test:launcher && npm run test:scripts && npm run verify-version && npm run check:docs && npm run check:validate && npm run check:blueprint && npm run coverage:check && cargo deny check licenses bans sources`.

## Work blocks

### Block 1: Split MCP retrieval from source resolution

- [ ] Done.
- Dependencies: none.
- Areas: `contract-003-api-sokf`, `crates/lib/superdev-core/src/sokf/mcp.rs`, `crates/lib/superdev-core/src/sokf/mutation.rs`, `crates/lib/superdev-core/src/sokf/mod.rs`, `crates/lib/superdev-core/tests/mcp_tools.rs`, `scripts/test/fixtures/sokf-mcp-fake.mjs`, and `scripts/test/sokf-mcp-client.test.mjs`.
- Outcome: `sokf_resolve_source` returns a canonical physical target with generated authority and its mirrored text item, `sokf_retrieve` owns semantic overview, concept, and section behavior, `sokf_read` is absent from both the tool list and the server instructions, and the MCP server binds itself to the canonical active checkout root.
- Verification: `cargo test -p superdev-core sokf && cargo test -p superdev-core --test mcp_tools && node --test scripts/test/sokf-mcp-client.test.mjs`.
- Tests: table-driven MCP cases cover `contract-003-api-sokf AC_resolve-schema`, `AC_retrieve-schema`, `P_active-worktree-root`, `AC_worktree-other-checkout-refused`, `P_source-resolution`, `AC_source-identity`, `AC_source-contained`, `AC_source-section-refused`, `P_semantic-retrieve`, `AC_semantic-addresses`, `AC_semantic-physical-refused`, `P_no-read-alias`, `AC_instructions-name-served-tools`, `P_direct-does-not-load`, `P_first-index-call-loads`, `P_direct-retrieval-skips-index`, `P_parse-error-quoted`, and `P_overview-warning-cap`. Tool-list schemas assert exact required fields, optional fields, integer minima, and `additionalProperties: false`; one case asserts that a successful `sokf_resolve_source` result carries exactly one text item whose bytes equal the pretty-printed JSON of its structured content; one case asserts the `get_info()` instruction string names each served tool and contains neither `sokf_read` nor a directed read of the `sokf:` overview address; runtime cases reject one unknown property per request and each missing required property. An existing file returns its canonical repository-relative path and `exists: true`; a missing file below an existing contained ancestor returns the normalized destination and `exists: false`; an existing file symlink and a missing path below a directory symlink canonicalize to contained targets; file and directory symlinks that land elsewhere inside the repository canonicalize successfully; file and directory symlinks that escape the repository fail before read; a missing suffix below contained and escaping symlinks respectively succeeds and fails; and a generated projection returns exact inclusive lines, authoritative path, and region.
- Structural evidence: every resolver matrix row asserts the ingress path, canonical target, `exists`, exact `generatedRegions`, and error or filesystem non-change as applicable. Contained and escaping `..` normalization cases respectively remain inside the repository and fail before access. A real Git fixture creates a main checkout and a linked worktree on divergent branches with different bytes for the same concept identity; calls from the worktree root and a nested directory resolve and retrieve locally, and a path into the other checkout fails before access. The MCP tool-list assertion contains `sokf_resolve_source` and `sokf_retrieve` but not `sokf_read`; schema snapshots equal the contract field-for-field, and the resolver result snapshot equals the contract's declared structured content and mirrored text item field-for-field. The instruction-string assertion reads the value `get_info()` returns rather than a copy of it. `cargo run -- validate` proves the contract Definition matches its marked source regions and that every Block 1 `PENDING(issue-077)` marker is removed while each issue-082 and issue-083 marker remains.
- Documentation: implement and materialize the scoped `contract-003-api-sokf` declarations, then run `cargo run -- validate --fix` followed by `npm run check:validate`.

### Block 2: Route Pi's built-in read and verify the repository

- [ ] Done.
- Dependencies: Block 1.
- Areas: `contract-012-api-sokf-pi-file-tools` including its new marked `tools` region, `.pi/extensions/sokf.ts`, `.pi/extensions/sokf-mcp.ts`, `scripts/test/fixtures/sokf-pi-adapter-smoke.ts`, `scripts/test/sokf-pi-adapter.test.mjs`, `/README.md`, `/CONTRIBUTING.md`, `/CHANGELOG.md`, `/knowledge/contracts/index.md`, `architecture`, `software-components`, `testing-strategy`, `security-requirements`, and `development-commands`.
- Outcome: routed Pi read behavior matches the paired built-in exactly, the adapter selects the canonical active checkout root, public and canonical documentation describe the implemented behavior, and focused plus complete suites pass.
- Verification: `cargo test -p superdev-core sokf && cargo test -p superdev-core --test mcp_tools && node --test scripts/test/sokf-pi-adapter.test.mjs && node --test scripts/test/sokf-mcp-client.test.mjs && npm run check:docs && npm run check:validate && npm run check:blueprint`.
- Tests: paired adapter cases cover `contract-012-api-sokf-pi-file-tools P_source-routing`, `AC_source-existing`, `AC_source-missing`, `AC_source-nested-cwd`, `AC_source-pi-preparation`, `AC_source-contained-symlink`, `AC_source-escaping-symlink`, `P_read-parity`, `AC_read-exact-source`, `AC_read-semantic-refused`, `AC_read-description-accurate`, `AC_worktree-discovery`, `AC_worktree-git-precedence`, `AC_worktree-cross-checkout-refused`, and `P_pinned-pi`. Paired read cases cover success, access and out-of-range failures, 1-indexed offset and limit, 2,000-line and 50 KB truncation, one oversized line, continuation text, cancellation, exact details and errors, and renderer inputs. Path-preparation cases prove leading `@`, `~`, Unicode-space, macOS AM/PM, NFD, curly-quote, and combined variants resolve exactly as Pi does. Repository-root and nested-working-directory spellings resolve identically. The refusal message names `sokf_search`, and the registered `read` description advertises no refused address form.
- Structural evidence: the paired test imports the pinned Pi 0.85.1 test dependency rather than a `pi` executable on `PATH`, fails when that dependency cannot load, and never skips. The path-preparation adapter is proven byte-for-byte against the values observed at Pi 0.85.1 operation boundaries, and no SOKF adapter source or SOKF parity test deep-imports unexported package internals. A routed excerpt copied with frontmatter is byte-identical to the same source range read by the built-in tool. A linked-worktree fixture proves the adapter selects the worktree root from its `.git` pointer file, reads only that checkout's bytes, and rejects a path into the main checkout. A fixture whose tree carries `.superdev/config.toml` and no `.git` marker still resolves and routes, and a fixture carrying both markers at different depths selects the `.git` ancestor. Existing routed edit and write cases still pass unchanged. `cargo run -- validate` runs twice with a clean second pass and proves every Block 2 `PENDING(issue-077)` marker is removed while each issue-082 and issue-083 marker remains.
- Documentation: update `/README.md`, `/CONTRIBUTING.md`, `/CHANGELOG.md`, and the canonical knowledge listed above, run `cargo run -- validate --fix` twice, then run `npm run check:docs`, `npm run check:validate`, and `npm run check:blueprint`.

## Build state

Current block: 1. Attempts: 0. Final corrections: 0. Blocker: scope requirements review and human approval pending.

## Implementation decisions

none. BUILD records only local choices that do not change the approved contracts or ADR.

## Follow-up issues

Routed mutation parity and the persistence outcome boundary remain in
[issue-082][sokf:issue-082-sokf-mutation-parity]. Validation follow-up delivery,
file-tool prompt metadata, and removal of `/SOKF-EDIT-RELIABILITY-PLAN.md` remain
in [issue-083][sokf:issue-083-sokf-validation-follow-ups]. This plan leaves
`sokf_retrieve` an MCP-only operation, so a Pi session reaches canonical
knowledge through exact source, `sokf_search`, and `sokf_graph`; registering a
Pi-side retrieval tool for rendered concepts and the `sokf:` overview requires a
separate issue. SOKF membership through contained symlinks was declined in
[issue-081][sokf:issue-081-sokf-symlink-membership]; resolution accepts a
contained target and refuses an escaping one, which this plan already covers. The
general validator symlink walk remains in issue-031. Any change to semantic
ranking, rendered retrieval content, MCP transport lifecycle, search, or graph
behavior requires a separate issue.

## Completion evidence

Scope requirements review and explicit human approval are pending. The parent
workflow must publish the approved knowledge-only scope as the immutable SCOPE
checkpoint before BUILD starts.

Workflow default branch: main.

<!-- sokf:links -->
[sokf:coding-standards]: /knowledge/coding-standards.md
[sokf:issue-077-sokf-file-tool-parity]: /knowledge/issues/open/issue-077-sokf-file-tool-parity.md
[sokf:issue-081-sokf-symlink-membership]: /knowledge/issues/wontfix/issue-081-sokf-symlink-membership.md
[sokf:issue-082-sokf-mutation-parity]: /knowledge/issues/open/issue-082-sokf-mutation-parity.md
[sokf:issue-083-sokf-validation-follow-ups]: /knowledge/issues/open/issue-083-sokf-validation-follow-ups.md
