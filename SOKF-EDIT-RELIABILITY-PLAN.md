# Pi File-Tool Parity Plan for SOKF

**Status:** Proposed, outside the Superdev/SOKF workflow

## Objective

Make SOKF-routed `read`, `edit`, and `write` behave exactly like Pi's built-in
file tools. A caller should observe the same parameter contract, argument
preparation, matching behaviour, errors, cancellation, truncation, result
content, `details`, rendering, and context cost whether a path is physical or
resolved from `sokf:<id>`.

SOKF adds only routing plus post-mutation repair and validation. It must not
invent a second file-tool UX or return internal mutation reports to the LLM.
Semantic knowledge retrieval remains the job of `sokf_search` and `sokf_graph`;
`read` remains a file read even when its path is a virtual SOKF identity.

## Confirmed problem

- `sokf:<id>` edit routing works, but failures occur when `oldText` is stale or
  reconstructed from retrieval-oriented output instead of exact source.
- `crates/lib/superdev-core/src/sokf/mutation.rs::whole_file_diff` serializes
  every old line as removed and every new line as added.
- `.pi/extensions/sokf.ts` returns that MCP mutation report in tool-result
  `content`, so the entire report enters the LLM context. It also converts the
  patch into `EditToolDetails`, duplicating substantial data in the persisted
  result.
- Pi's built-in edit instead returns one concise sentence to the LLM and places
  a compact display diff plus unified patch in `EditToolDetails`.
- The current virtual read returns rendered knowledge rather than exact file
  bytes, unlike Pi's built-in read.
- The current SOKF write result exposes mutation internals rather than matching
  Pi's concise built-in write result.
- The extension rewrites the file tools' prompt snippets and distributes one
  SOKF decision across several `read`, `edit`, `write`, and `sokf_search`
  guidelines. The generated `Available tools` and `Guidelines` sections are
  therefore repetitive and blur file operations with knowledge discovery.

## Reference contract

Treat the installed Pi factories as the executable specification:

- `createReadTool()` and `ReadToolDetails`
- `createEditTool()` and `EditToolDetails`
- `createWriteTool()`
- `generateDiffString()` and `generateUnifiedPatch()`
- `withFileMutationQueue()`

Do not maintain approximate copies of built-in behaviour. Reuse these factories
and exported helpers wherever possible. An override must preserve the exact
result shape because Pi's renderer and session machinery depend on it.

## Non-negotiable parity

### Read

For identical file bytes and arguments, physical and SOKF-routed reads must
return equivalent built-in results:

- exact source bytes decoded as UTF-8, not a semantic summary;
- the same 1-indexed `offset` and `limit` behaviour;
- the same 2,000-line/50-KB head truncation and continuation messages;
- the same `ReadToolDetails.truncation` shape;
- the same out-of-range and access errors;
- the same abort behaviour;
- the inherited built-in renderer, prompt snippet, and prompt guidelines.

`read path="sokf:<id>"` resolves the identity and then reads its exact source.
Section-qualified reads must either map deterministically onto exact source
lines while preserving built-in pagination semantics or move to a separate
semantic operation; they must not make the file `read` tool return a different
result dialect.

### Edit

For equivalent original bytes and edits, physical and SOKF-routed edits must
match Pi's built-in edit in:

- current `edits[]` schema and legacy argument preparation;
- non-empty edit validation;
- matching against one original snapshot;
- Pi's exact-then-normalized matching behaviour;
- uniqueness and overlap checks;
- BOM and line-ending preservation;
- abort and mutation-queue behaviour;
- concise LLM content:
  `Successfully replaced N block(s) in PATH.`;
- `EditToolDetails` containing the same compact display diff, standard unified
  patch, and `firstChangedLine` semantics;
- thrown-error behaviour and wording where the built-in contract applies;
- inherited built-in rendering, including preview and expansion behaviour.

SOKF policy may reject an otherwise valid built-in edit—for example an `id`,
`verified`, or generated-projection mutation—but the rejection must be concise,
actionable, and signalled as a real tool error. It must never suggest or
silently perform a whole-file-write fallback.

### Write

For equivalent input, physical and SOKF-routed writes must match Pi's built-in
write in:

- `{ path, content }` parameters;
- recursive parent creation;
- abort and mutation-queue behaviour;
- concise LLM content: `Successfully wrote to PATH`;
- `details: undefined`;
- thrown errors;
- inherited renderer, prompt snippet, and guidance that write is for new files
  or deliberate complete rewrites only.

SOKF repair and validation may run after the write, but their internal reports
must not replace the built-in result shape.

## Architecture

### 1. Resolve before delegation

Add one narrow SOKF resolution/source operation that returns:

- the canonical repository-relative physical path;
- exact source bytes or a way for the adapter to read them;
- whether the destination exists;
- enough authority metadata to enforce protected/generated regions.

Resolve `sokf:<id>` before executing a built-in-compatible tool. Use the real
physical target as the mutation queue key so virtual and physical aliases
serialize together.

### 2. Delegate to Pi rather than emulate Pi

Refactor `.pi/extensions/sokf.ts` so each override begins with the corresponding
Pi factory and retains its metadata and renderers. Override only the operations
needed to route SOKF paths:

- read operation: supply exact resolved bytes;
- edit operations: read exact bytes and submit the computed replacement through
  agent-safe SOKF mutation;
- write operations: submit content through SOKF mutation and let SOKF create
  parents as required.

If a factory cannot be used directly because virtual path resolution would
produce the wrong queue key or displayed path, extract a small parity adapter
around the factory. Verify its outputs byte-for-byte against the factory; do not
fork Pi's matching, truncation, diff, or rendering algorithms.

### 3. Keep SOKF lifecycle data out of model content

Capture MCP mutation details extension-side for:

- setting `knowledgeMutated`;
- final turn validation;
- diagnostics and optional TUI-only inspection;
- repair/refile bookkeeping.

Return only the same `content` the built-in tool would return. Do not put full
mutation JSON, repair patches, validation dumps, or duplicate diffs into the
LLM-visible tool result. If detailed diagnostics need persistence or display,
store them in non-context session entries or bounded tool `details` only where
that exactly matches the built-in type.

### 4. Use Pi's diff implementation

For edit results, use Pi's exported `generateDiffString()` and
`generateUnifiedPatch()` on the before/after source associated with the
requested target. Do not feed `whole_file_diff()` into `EditToolDetails`.

Decide explicitly how automatic repair or refiling after the requested edit is
reported. The normal result must still match Pi's edit shape. Additional repair
information should be a concise notification or TUI-only record, never an
unbounded second patch in LLM content.

### 5. Preserve SOKF safety as a routing concern

Continue enforcing:

- stable `id`;
- byte-preserved `verified`;
- no stamped fields in the working tree;
- schema repair, index repair, refiling, and validation;
- generated include ownership.

Detect edits intersecting generated projections before mutation. Reject with
the authoritative source path instead of allowing repair to erase the change.

### 6. Keep system-prompt responsibilities distinct

The solution is better wording, not more wording.

- Let `read`, `edit`, and `write` retain Pi's built-in prompt snippets and
  baseline guidelines exactly once. Their `Available tools` entries describe
  file operations, not SOKF policy.
- Let `sokf_search` and `sokf_graph` describe knowledge discovery and traversal.
- Add only the minimum routing rules that Pi's built-ins cannot know.
- Prefix every custom guideline with the tool it governs, because Pi appends
  custom guidelines into one flat list.
- State each decision once; do not repeat the known-ID/search distinction under
  both `read` and `sokf_search`.

Use this target wording as the baseline, adjusting only if Pi's inherited text
changes:

```text
Available tools additions:
sokf_search: Search canonical project knowledge by meaning
sokf_graph: Follow relationships between canonical knowledge concepts

SOKF-specific guidelines:
sokf_search: use it when no concept ID is known; otherwise read sokf:<id>.
edit: use sokf:<id> for an existing knowledge concept; section addresses are read-only.
write: create a knowledge concept at its physical knowledge/...md path.
```

Do not override the built-in `read`, `edit`, or `write` snippets merely to
advertise routing. Their schemas and the concise rules above already expose the
SOKF path conventions.

## Work plan

### Phase 1 — Characterization and parity harness

1. Add a test harness that executes each Pi built-in tool and the SOKF-routed
   override against paired temporary files.
2. Normalize only unavoidable target-path differences; compare all other
   `content`, `details`, errors, and filesystem bytes exactly.
3. Cover success, failure, abort, offset/limit, truncation, BOM, CRLF, Unicode,
   fuzzy-normalized matching, duplicate matches, overlapping edits, multiple
   disjoint edits, creation, overwrite, and nested parent creation.
4. Assert that no MCP mutation JSON or whole-file patch appears in LLM-visible
   `content`.
5. Record tool-result byte/token size for a one-line edit to a large concept and
   set the expected result to the same order of magnitude as built-in edit.

### Phase 2 — Read parity

1. Add exact-source resolution to the SOKF MCP/service boundary.
2. Route virtual reads through `createReadTool()` operations.
3. Preserve built-in offset, limit, truncation, continuation, error, abort,
   details, and rendering behaviour.
4. Move retrieval-only formatting away from `read` where necessary.
5. Add contract tests proving copied read text is valid `oldText` for edit,
   including frontmatter.

### Phase 3 — Edit parity

1. Resolve the real target and queue on that path.
2. Reuse built-in argument preparation and edit semantics rather than the
   current independent exact-edit implementation at the Pi boundary.
3. Apply the resulting content through agent-safe SOKF mutation.
4. Build result details with Pi's own diff helpers and return Pi's concise
   success content.
5. Throw concise errors, preserving built-in wording for built-in failures and
   adding only necessary SOKF-policy errors.
6. Remove the MCP envelope and whole-file report from the returned `content`.

### Phase 4 — Write parity

1. Route SOKF writes through `createWriteTool()`-compatible operations.
2. Match parent creation, abort, queue, error, content, details, and renderer
   behaviour.
3. Keep repair/validation state extension-side.
4. Verify creation by physical `knowledge/...` path and overwrite by supported
   virtual/physical paths without weakening the guidance against whole-file
   rewrites.

### Phase 5 — Repair reporting and generated content

1. Define a bounded, non-context channel for repair/refile diagnostics.
2. Ensure final validation follow-ups contain only actionable findings, not
   complete patches.
3. Reject generated-projection edits with their authoritative source path.
4. Confirm a repair touching additional files does not change the standard
   read/edit/write result shapes.

### Phase 6 — Clarify system-prompt metadata

1. Spread or copy the Pi factory metadata so `read`, `edit`, and `write` retain
   their built-in snippets and guidelines without paraphrase or duplication.
2. Replace the current SOKF prompt metadata with the concise target wording in
   this plan.
3. Test the generated `Available tools` and `Guidelines` sections only if this
   can be done as a fast, in-process unit test over tool metadata—no Pi process,
   MCP server, repository fixture, filesystem setup, or model call. Otherwise,
   omit the snapshot and assert directly against the registered snippets and
   guideline arrays.
4. Keep any snapshot narrowly scoped to those two sections, deterministic, and
   cheap enough for the normal unit-test loop; delete it if it becomes an
   integration test.
5. Assert that:
   - file-tool descriptions discuss file operations;
   - SOKF discovery is described by `sokf_search` and `sokf_graph`;
   - every added flat guideline names its tool;
   - known-ID lookup, existing-concept editing, and new-concept creation each
     appear once;
   - no rule recommends a whole-file fallback.
6. Review the tested text as prose: shorter is preferred, and no added sentence
   may restate an inherited Pi rule.

### Phase 7 — Remove duplicate implementations

1. Remove `displayDiff()` and `editDetails()` from `.pi/extensions/sokf.ts` once
   Pi's helpers/factories own those outputs.
2. Stop returning `envelope.content` from SOKF-routed mutations.
3. Remove or narrow `whole_file_diff()` if no non-Pi client requires it. If the
   MCP contract retains patches, generate compact contextual patches and bound
   them independently.
4. Remove any prompt metadata made redundant by inherited Pi metadata or the
   three SOKF routing rules.

## Primary files

- `.pi/extensions/sokf.ts`
- `.pi/extensions/sokf-mcp.ts`
- `crates/lib/superdev-core/src/sokf/mcp.rs`
- `crates/lib/superdev-core/src/sokf/mutation.rs`
- `crates/lib/superdev-core/tests/mcp_tools.rs`
- `scripts/test/fixtures/sokf-pi-adapter-smoke.ts`
- `scripts/test/sokf-pi-adapter.test.mjs`
- `.pi/skills/sokf-authoring/SKILL.md`
- any shipped/materialized copies and lock entries required by packaging

## Verification

Run at minimum:

```text
cargo test -p superdev-core sokf
cargo test -p superdev-core --test mcp_tools
node --test scripts/test/sokf-pi-adapter.test.mjs
node --test scripts/test/sokf-mcp-client.test.mjs
cargo run -q -p superdev -- validate
```

Also run the repository's full Rust and JavaScript suites. Manually compare
physical and `sokf:<id>` calls for all three tools in Pi, including expanded TUI
rendering and the resulting session JSONL.

## Acceptance criteria

- SOKF-routed read, edit, and write pass the paired built-in parity harness.
- `read sokf:<id>` returns exact mutation-ready source with built-in pagination
  and truncation semantics.
- A one-line SOKF edit returns one concise sentence in LLM content and the same
  compact `EditToolDetails` shape as built-in edit.
- The one-line edit does not inject a whole-file patch or mutation JSON into LLM
  context.
- SOKF write returns the same concise content and undefined details as built-in
  write.
- Virtual and physical aliases share the same mutation queue.
- SOKF safety, repair, refiling, and validation remain enforced without changing
  standard tool result shapes.
- Generated projections direct the agent to authoritative source instead of
  accepting a transient edit.
- Existing concepts are never rewritten wholesale as automatic recovery from a
  targeted-edit failure.
- The generated `Available tools` section leaves file operations under Pi's
  built-in `read`, `edit`, and `write` wording and reserves SOKF wording for
  knowledge discovery/traversal.
- The generated `Guidelines` section contains only the three concise SOKF
  routing rules, each names its tool, and no rule duplicates inherited Pi
  guidance.
- Prompt-text coverage is a fast in-process unit test; if producing a snapshot
  requires integration setup, direct metadata assertions are used instead.
