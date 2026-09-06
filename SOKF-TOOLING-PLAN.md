# SOKF Tooling and Context Plan

**Status:** Implemented through behavioral fixture definition; model-session scoring and non-overriding harness adapters remain.

## Objective

Make the Superdev Open Knowledge Format (SOKF) knowledge the default source of project truth for an agent without loading the SOKF specification or the full knowledge tree into every model context.

The agent must always know that SOKF exists and reach for it when project knowledge could answer the task. The agent must write project knowledge to SOKF and update affected concepts when the project changes. Files outside the SOKF knowledge may carry outward-facing documentation, but must not become a second internal knowledge store. The agent must load only the relevant concepts during ordinary work. It must load the full specification only when the task depends on format semantics.

## Design principles

1. Keep SOKF awareness in the always-loaded instructions.
2. Load SOKF content through progressive retrieval.
3. Make the Superdev command-line interface (CLI) the portable interface.
4. Make harness extensions adapt the CLI or its underlying service instead of reimplementing SOKF behavior.
5. Reuse coding-tool conventions for common read, edit, and write operations.
6. Keep semantic search and graph traversal explicitly SOKF-specific.
7. Keep one domain implementation behind the CLI, Model Context Protocol (MCP), and harness adapters.
8. Write newly established project knowledge to SOKF and update existing concepts when their subjects change.
9. Keep internal project knowledge out of files outside the SOKF knowledge; allow outward-facing documentation there without duplicating its full content in SOKF.
10. Measure whether agents use and maintain SOKF at the right times instead of relying only on prompt wording.

## Target agent interaction

The standing instruction should be approximately:

> SOKF is the canonical store for all project knowledge. Use it whenever project knowledge is needed. Read concepts using `sokf:<id>`. Write new project knowledge there and keep it current. Only outward-facing project information belongs outside SOKF; summarize and cite that information in SOKF instead of duplicating it.

A Pi agent should be able to use:

```text
read        path="sokf:architecture"
edit        path="sokf:architecture"
write       path="knowledge/new-concept.md"
sokf_search query="authentication flow"
sokf_graph  id="architecture"
```

The common operations use familiar coding tools. Only semantic search and graph traversal require specialized tools.

The portable CLI must expose the same read, edit, and write operations. Harness extensions must not own mutation semantics that other agents cannot use.

### Virtual SOKF addresses

Use an explicit `sokf:` namespace to avoid ambiguity with filesystem paths.

| Address | Meaning |
| --- | --- |
| `sokf:` | Overview of the SOKF knowledge |
| `sokf:<id>` | Whole concept identified by `<id>` |
| `sokf:<id>#<heading>` | One section identified by a heading or heading path |

Example:

```text
read path="sokf:architecture#Runtime > Startup"
```

Section addresses use `#`; heading paths retain the rendered `A > B` form.

## Workstream 1: Portable SOKF CLI

The existing CLI exposes `superdev sokf index`. Add retrieval and mutation commands:

```text
superdev sokf overview
superdev sokf search <query>
superdev sokf read <id-or-path>
superdev sokf edit <id-or-path>
superdev sokf write <id-or-path>
superdev sokf graph [id]
```

Proposed retrieval calls:

```bash
superdev sokf overview
superdev sokf search "authentication flow"
superdev sokf search "authentication flow" --type ADR --tag security --limit 10
superdev sokf read architecture
superdev sokf read architecture --heading "Runtime > Startup"
superdev sokf read architecture --offset 40 --limit 30
superdev sokf graph architecture
```

Proposed mutation calls should support both human-friendly arguments and request JSON from a named file or standard input. `--request-json -` means standard input:

```bash
superdev sokf edit architecture --old-text "old" --new-text "new"
superdev sokf edit architecture --allow-restricted --old-text "old" --new-text "new"
printf '%s' '{"path":"sokf:architecture","edits":[{"oldText":"old","newText":"new"}]}' \
  | superdev sokf edit --request-json -

superdev sokf write knowledge/new-concept.md --content-file draft.md
superdev sokf write knowledge/new-concept.md --content-stdin < draft.md
printf '%s' '{"path":"knowledge/new-concept.md","content":"---\ntype: Module\n---\n"}' \
  | superdev sokf write --request-json -
```

The machine request shapes must match the coding tools so a harness adapter can forward existing arguments unchanged:

Read:

```json
{ "path": "sokf:architecture", "offset": 1, "limit": 200 }
```

Edit:

```json
{
  "path": "sokf:architecture",
  "edits": [
    { "oldText": "unique original text", "newText": "replacement text" }
  ]
}
```

Write:

```json
{
  "path": "knowledge/new-concept.md",
  "content": "complete concept document"
}
```

`edit` must use Pi-compatible exact replacement semantics.

Every `oldText` must match exactly once in the original file. Multiple edits must be non-overlapping and evaluated against the same original content. The command must make no change if any edit is invalid.

`write` must use Pi-compatible whole-file replacement semantics.

For an existing concept, `edit` and `write` may accept an ID or a path. Creation must name a physical path because an ID does not determine placement. An unresolved target must not silently become a new path.

Each command should support stable machine-readable results:

```bash
superdev sokf read architecture --json
superdev sokf edit --request-json request.json --json
```

### Shared service

Extract the domain behavior currently held by the MCP handlers in `crates/lib/superdev-core/src/sokf/mcp.rs` into a harness-independent service:

```rust
pub struct SokfService { /* ... */ }

impl SokfService {
    pub fn overview(/* ... */) -> Result</* ... */>;
    pub fn search(/* ... */) -> Result</* ... */>;
    pub fn read(/* ... */) -> Result</* ... */>;
    pub fn edit(/* ... */) -> Result</* ... */>;
    pub fn write(/* ... */) -> Result</* ... */>;
    pub fn graph(/* ... */) -> Result</* ... */>;
    pub fn resolve(/* ... */) -> Result<ConceptIdentity>;
}
```

Use the service from thin adapters:

```text
CLI ──┐
      ├── SokfService
MCP ──┘
```

The service should own resolution, index synchronization, filtering, limits, near-miss errors, and rendering or structured results. The adapters should own argument parsing and transport-specific result encoding.

### CLI deliverables

- Add `overview`, `search`, `read`, `edit`, `write`, and `graph` subcommands.
- Preserve the existing MCP retrieval behavior while adding mutation tools.
- Return equivalent retrieval information through the CLI and MCP.
- Give `edit` atomic, exact-replacement semantics compatible with Pi's edit tool.
- Give `write` explicit whole-file replacement semantics compatible with Pi's write tool.
- Apply agent-safe restrictions by default and provide an explicit CLI-only human override.
- Run automatic repair after each successful mutation.
- Resolve existing concepts by ID without making new-concept placement implicit.
- Return the final physical path plus requested-mutation and automatic-repair diffs.
- Support text output for humans and JSON output for adapters.
- Document exit codes, malformed input, no-match, multiple-match, overlap, and validation behavior.
- Test CLI and MCP retrieval parity at the shared-service boundary.
- Test that failed mutations leave the target byte-identical.

## Workstream 2: Minimal Pi extension

Build a Pi extension that invokes the CLI or a stable machine interface exposed by the CLI. Do not duplicate parsing, retrieval, ranking, or graph behavior in TypeScript.

### Override `read`

Create Pi's original read tool with `createReadTool()`. Delegate ordinary paths unchanged:

```ts
if (!path.startsWith("sokf:")) {
    return originalRead.execute(/* ... */);
}
```

Route virtual SOKF addresses through the Superdev CLI:

| Pi input | Superdev operation |
| --- | --- |
| `sokf:` | `sokf overview` |
| `sokf:<id>` | `sokf read <id>` |
| `sokf:<id>#<heading>` | `sokf read <id> --heading <heading>` |
| `offset` and `limit` | A line window over the rendered concept |

Keep the built-in schema:

```ts
{
  path: string,
  offset?: number,
  limit?: number
}
```

The override must preserve Pi's exact read result and detail shapes. This preserves built-in rendering, truncation reporting, and session behavior.

### Register `sokf_search`

Retain the existing conceptual argument shape:

```ts
{
  query: string,
  limit?: number,
  types?: string[],
  tags?: string[],
  lifecycle?: string[]
}
```

Use the tool description to reinforce the standing rule:

> Search SOKF whenever project knowledge is needed.

The extension should call `superdev sokf search ... --json` and render the stable result.

### Register `sokf_graph`

Keep graph traversal separate because it has no natural file-tool equivalent. The tool accepts an optional concept ID and returns either that concept's neighborhood or the complete edge map.

### MCP and fallback adapters

Expose five MCP tools with familiar request semantics. `sokf_read` uses a path and optional line window; `sokf:` is the overview address. Search and graph remain specialized, while mutations match coding tools:

```text
sokf_read  { path, offset?, limit? }
sokf_edit  { path, edits }
sokf_write { path, content }
```

Both tools use the same `SokfService` mutation operations as the CLI, apply agent-safe restrictions, run automatic repair, and return every changed path and diff. MCP does not expose the human restriction override.

This changes the MCP server from read-only to mutation-capable. Update its tool descriptions, server instructions, API contract, security documentation, and tests accordingly. The server still trusts its local standard-input client and receives no new filesystem authority beyond the Superdev process.

The separate `sokf_overview` tool is removed while the API is unreleased. Pi exposes virtual reads through its familiar `read` override; harnesses that cannot override built-in tools can call the five namespaced MCP tools.

## Workstream 3: Pi mutation adapters

Implement the portable CLI mutation operations before adding Pi overrides. Pi must delegate SOKF mutation semantics to those commands rather than resolve and mutate concepts independently.

### Override `edit`

Support an existing concept as an edit target:

```text
edit path="sokf:architecture"
```

For ordinary paths, delegate unchanged to Pi's original `edit` tool. For a `sokf:` address, forward the existing Pi request unchanged to `superdev sokf edit --request-json - --json`.

The CLI performs ID resolution and exact replacements. The extension converts the CLI result into Pi's exact `EditToolDetails` shape so normal diff rendering and session behavior remain intact.

The extension cannot learn the resolved physical path from the mutation result and then queue an operation that has already happened. Route every SOKF mutation, including physical paths inside the configured knowledge root, through one queue keyed by the canonical knowledge root. This serializes ID-addressed and path-addressed SOKF mutations without a resolve-then-mutate race. Ordinary paths outside that root continue to use Pi's normal per-file queues.

Reject section-qualified mutation targets such as `sokf:architecture#Runtime`. Sections are suitable retrieval units but ambiguous mutation targets.

### Override `write`

Support both existing concepts and new physical paths:

```text
write path="sokf:architecture" content="...complete document..."
write path="knowledge/new-concept.md" content="...complete document..."
```

For ordinary paths outside the SOKF knowledge, delegate unchanged to Pi's original `write` tool. Forward an existing `sokf:<id>` target and every physical path inside the SOKF knowledge unchanged to `superdev sokf write --request-json - --json`.

The CLI performs ID resolution and whole-file replacement. The extension converts the result into Pi's exact write result shape and uses the knowledge-root mutation queue described above.

An unresolved `sokf:<id>` target is an error. New concepts must name a physical path. A future schema-aware creation command may choose placement, but `write` must not invent it.

### Mutation policies

SOKF mutations use one of two explicit policies:

- **Agent-safe** is the default for the CLI and mandatory for MCP and harness tools. It rejects changes to an existing concept's `id`, preserves `verified` byte-for-byte, and rejects stamped fields.
- **Human override** allows a deliberate CLI caller to change an existing `id` or `verified`. Expose it as a CLI-only flag such as `--allow-restricted`; do not include it in the machine request schema or MCP tools. Stamped fields remain invalid because SOKF forbids them in the working tree for every actor.

Both policies must:

- reject targets outside the configured knowledge root, including escapes through `..` or symlinks;
- define how reserved files such as `index.md` and `manifest.sokf.yaml` may be changed;
- write atomically without replacing a symlink outside the knowledge root;
- leave the original file unchanged when an input precondition fails.

Represent the policy explicitly in the shared Rust API rather than as scattered booleans:

```rust
enum MutationPolicy {
    AgentSafe,
    HumanOverride,
}
```

### Automatic repair and validation

A successful `edit` or `write` automatically runs the equivalent of `superdev validate --fix` while holding the knowledge-root mutation queue. This permits the mutation API to regenerate definition blocks, refresh includes, and refile concepts instead of requiring the agent to edit generated content.

Each mutation result must return:

- the originally resolved target;
- its final path after repair or refiling;
- the diff for the requested mutation;
- every additional path and diff changed by automatic repair;
- the remaining validation findings.

Run one final `superdev validate` at the end of a turn that performed mutations. Return remaining findings to the model and preserve each mutation and repair diff in the session.

Input-precondition failures make no changes. Once the requested mutation has been applied, repair or validation failures do not roll it back. SOKF may remain temporarily invalid between related mutations, such as when one concept links to a concept that the next mutation creates.

Return an applied mutation with explicit state rather than a transport or tool-execution error:

```json
{
  "applied": true,
  "validation": "invalid",
  "changes": [],
  "findings": [
    { "severity": "error", "message": "link target `new-concept` does not exist" }
  ]
}
```

This distinction prevents a harness from retrying an edit that already happened. Use a hard tool error only when no requested mutation was applied. If an input/output failure interrupts automatic repair after it changed additional files, return `applied: true`, list every known change, and set `validation` to `unknown`. Never claim rollback unless the implementation completed and verified one.

The end-of-turn validation is the convergence boundary: intermediate states may be invalid, but the agent must continue until final validation passes. The design must define how final findings trigger another agent turn without creating an unbounded retry loop.

## Workstream 4: Always-on awareness

Keep the always-loaded SOKF instruction concise. It belongs in the Superdev agent instructions rather than an on-demand skill.

Candidate instruction:

```xml
<knowledge>
SOKF under `knowledge/` is the canonical store for all project knowledge.
Use it whenever project knowledge is needed. Read concepts as `sokf:<id>`.
Write new project knowledge there and keep it current.
Only outward-facing project information belongs outside SOKF. Summarize and
cite that information in SOKF instead of duplicating it.
Use the SOKF authoring instructions when writing concepts. Read
`.agents/sokf/SPEC.md` only when the task depends on format semantics.
</knowledge>
```

Tool descriptions should reinforce this instruction without repeating the SOKF specification.

### SOKF authoring skill

An on-demand skill can govern format-sensitive work. It should cover:

- creating and indexing a schema before introducing a previously unknown `type`;
- frontmatter rules;
- stable identities;
- sources and claim attribution;
- typed links and body mirroring;
- generated definition and include blocks;
- restricted and stamped fields;
- validation and repair.

The skill should direct the agent to load `.agents/sokf/SPEC.md` when the task requires normative detail. Basic knowledge retrieval should not require loading the skill.

## Workstream 5: Evaluation

Add behavioral fixtures before expanding the standing prompt.

### Scenarios

1. **Known project question** — The agent searches SOKF before reading source files.
2. **Architecture decision** — The agent searches architecture decisions and conventions before proposing a design.
3. **Direct concept reference** — The agent uses `read` with `sokf:<id>`.
4. **Code-local task** — The agent reads explicitly named source directly without unnecessary SOKF retrieval.
5. **Concept edit** — The agent reads the concept schema and target, edits by ID, and validates.
6. **Multi-step knowledge change** — An early mutation creates a broken link or other temporary validation failure; later mutations complete the change and final validation passes.
7. **Knowledge-producing code change** — The agent updates affected concepts along with code and tests.
8. **New project decision** — The agent records the decision in SOKF rather than a root-level internal note.
9. **Outward-facing documentation** — The agent writes the external document outside SOKF and keeps only a concise, sourced summary in SOKF.
10. **SOKF format question** — The agent loads `SPEC.md`.
11. **Unknown concept ID** — The agent uses near misses or search rather than falling back to broad filesystem exploration.
12. **No SOKF knowledge** — The extension reports the missing store clearly and ordinary file tools continue working.

### Measurements

- Whether the agent consulted SOKF.
- Whether retrieval preceded broad code exploration.
- Whether the agent read only relevant concepts.
- Whether unrelated tasks avoided unnecessary SOKF calls.
- Whether format-sensitive tasks loaded the specification.
- Whether project knowledge was created or updated in SOKF.
- Whether internal knowledge leaked into ad hoc files outside SOKF.
- Whether outward-facing documentation was summarized and sourced rather than duplicated.
- Total always-loaded tokens.
- Number and sequence of retrieval calls.
- Success rate with and without the short standing instruction.

## Proposed implementation order

1. Define CLI text and JSON contracts for all six operations.
2. Extract `SokfService` from the MCP implementation.
3. Add the four read-only CLI commands.
4. Add CLI `edit` and `write` with automatic repair, agent-safe defaults, and the human override.
5. Add MCP `sokf_edit` and `sokf_write` using mandatory agent-safe policy and automatic repair.
6. Add service, CLI, and MCP retrieval parity tests plus mutation and repair tests.
7. Build the Pi extension with `sokf_search`, `sokf_graph`, and the `read` override.
8. Add Pi `edit` and `write` overrides that route SOKF targets through the CLI.
9. Replace verbose SOKF retrieval instructions with the concise standing instruction.
10. Add behavioral evaluations and refine the prompt from their results.
11. Add turn-end final validation and bounded repair feedback.
12. Add fallback adapters for harnesses that cannot override built-in tools.

## Initial recommendation

Adopt the hybrid interface:

- Extend `read`, `edit`, and `write` through the explicit `sokf:` namespace.
- Route physical writes inside the SOKF knowledge through the same safe CLI mutation layer.
- Retain specialized `sokf_search` and `sokf_graph` operations.
- Keep new concept creation path-based.
- Keep the full SOKF specification out of the default context.
- Keep the CLI and shared Rust service authoritative across harnesses.
- Expose `sokf_edit` and `sokf_write` over MCP with mandatory agent-safe policy.
- Run automatic repair as part of every SOKF mutation.
- Permit deliberate restricted-field and identity changes only through a CLI human override.

This approach gives agents familiar mechanics without presenting semantic search as ordinary text search or pretending a new concept's ID determines its path.

## Decisions made during implementation

- CLI adapter results use a `sokf-tools/v1` envelope with `content` text blocks
  and a `details` object. Retrieval leaves `details` empty; mutations put their
  applied state, validation state, paths, diffs, and findings there.
- The explicit CLI human override is `--allow-restricted`.
- Direct mutation permits reserved `index.md` navigation files but refuses
  `manifest.sokf.yaml`; neither is treated as an ordinary concept.
- Virtual section addresses use `sokf:<id>#<heading>`.
- Pi lazily starts one `superdev mcp sokf` process per repository and reuses it
  until session shutdown. The first search or overview read initializes the
  embedder; later index-dependent calls reuse that instance.
- The Pi adapter finds the nearest `.git` or `.superdev/config.toml` ancestor,
  treats its `knowledge/` directory as the routing boundary, and starts MCP or
  one-shot validation from that root. Physical mutation targets are forwarded
  as absolute paths, while virtual targets retain their `sokf:` address.
- Pi performs the narrow MCP initialize and `tools/call` exchange and rejects
  an incompatible MCP protocol version with a clear error.
- A Pi turn that applies a knowledge mutation runs final validation. A failed
  validation report triggers at most two automatic follow-up turns. Persistent
  failure is retained for the next manual turn and raises a user notification.
- Behavioral fixtures use `sokf-behavior/v1`. A structural Node test protects
  the 12-scenario roster; model-session execution and scoring remain separate.
- The script test suite loads the real adapter through Pi when Pi is available.
  A model-free sandbox checks MCP/adapter result parity, persistent process
  reuse, subdirectory routing, applied-invalid behavior, and the two-turn
  validation-feedback bound.
- Pi skills are independently authored under `.pi/skills`; Pi does not link the
  `.claude/skills` directory. Harness-specific skills may share SOKF policy but
  use each harness's native tool vocabulary and workflow conventions.
- CLI `read` applies `offset` and `limit` to the fully rendered concept after
  metadata and headings are present. Pi applies the same line-window semantics.
- Pi exposes overview and concept reads only through the `read` override; it
  does not retain `sokf_overview` or `sokf_read` aliases.
- Whole-document replacement of an existing concept remains available through
  `write path="sokf:<id>"`. Creation still requires a physical path.
- Agent-safe mutation owns checks that must prevent an unsafe or ambiguous
  write: containment and symlink safety, exact-edit preconditions, immutable
  identity, restricted fields, stamped fields, and reserved targets. Repair and
  full structural or referential validity remain validator responsibilities.
- The five MCP tools and six CLI operations are the fallback interface for
  harnesses that cannot override built-in tools. No further harness-specific
  adapter is added until a target harness requires one.
- Behavioral acceptance requires three trials per supported model: every
  safety-critical scenario must pass, at least 90% of mechanically scored
  assertions must pass overall, and no forbidden write may occur. The short
  standing-instruction arm must match or exceed the no-instruction arm on
  knowledge-worthy scenarios without causing the code-local scenario to
  consult SOKF.
- Direct reads and graph traversal parse current knowledge without loading an
  embedding model or touching the search index. Search and the overview address
  initialize the model once in a persistent MCP process, so later Pi calls do
  not repeat the roughly two-second startup.

## Open decisions

None. Model-session trials remain execution work against the acceptance rule
above rather than interface design.
