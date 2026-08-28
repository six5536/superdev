---
type: Spec
id: spec-cli-core-blueprint-engine
title: CLI Core & Blueprint Engine
description: Design for superdev's setup/management CLI — the manifest, the component model, and init/status/sync/update.
status: stable
links:
  - rel: relates-to
    to: architecture
    note: First concrete piece of the product architecture.
  - rel: relates-to
    to: software-components
    note: Extends the existing crate/binary/npm delivery shape.
---

# Summary

superdev is a Rust tool run inside target repos: `superdev init` sets a repo
up for agent-driven development, and further verbs keep that setup current.
It manages external components and natively provides AOKF — the canonical knowledge
format, its knowledge scaffolding, and (later) its MCP server with hybrid
search and structured update. The first managed repo is goodbye-tinnitus;
superdev is opinionated for this project's stack (Claude Code, mise, AOKF)
— generalisation is not a goal yet.

The product is built as four sub-projects, each with its own spec:

1. **CLI core + blueprint engine** — this spec.
2. **AOKF MCP server** — `superdev mcp aokf`: hybrid search (lexical +
   embeddings; local model default, API opt-in), graph traversal, structured
   update enforcing AOKF write classes. Subsumes porting the Python
   validator to Rust (`superdev aokf validate`).
3. **Skill pack plugin** — a Claude Code plugin (superpowers-style
   marketplace) carrying custom skills and MCP registrations.
4. **Knowledge upkeep + blueprint migrations** — folds into the
   `superdev aokf` verb family plus template-evolution machinery.

Decisions already made that bind later sub-projects: MCP querying replaces
wholesale preloading of the canonical knowledge in AGENTS.md; embeddings are pluggable
with a local model as default; existing per-repo skills (double-check,
grill-me, humanise, self-improve) are absorbed into the skill pack.

# Behaviour

- Stated in the design sections below rather than gathered here. This spec
  predates the contract that asks for one Behaviour section, and the
  sections were left where they were written rather than reshuffled after
  the fact.

# Acceptance criteria

Running `superdev init` in a fresh clone of a target repo produces a
committed, working agent-dev setup (mise pins installed, plugins registered,
codegraph indexed, canonical project knowledge scaffolded); `status` is clean afterwards and
exits 1 when a managed file or version drifts; `sync` restores it; a failed
apply leaves the repo as it started except for explicitly reported
irreversible steps. All existing CI gates stay green.

# Edge cases & errors

- Exit codes: `0` success; `1` work to do (`status` with drift); `2` usage
  error or hard failure. The broken-pipe-is-success rule stays.
- **Rollback.** Every applied action records its inverse in an in-run
  journal (file writes back up to `.superdev/cache/backup/<timestamp>/`
  first; plugin installs pair with uninstalls; TOML edits keep prior
  content). On failure the journal unwinds in reverse, best-effort; anything
  irreversible is listed explicitly as *not reverted*. The run ends with a
  per-component table — applied / reverted / not-reverted — and exits `2`.
  `sync` is the recovery path for whatever the unwind could not restore.
- External command failures are reported with the exact command line and
  verbatim stderr.
- If `claude` is missing from PATH, plugin steps fail soft: reported as
  skipped-with-reason, not a hard abort.

# Architecture

Three layers, in the existing workspace shape (see
[software-components](../software-components.md) and
[architecture](../architecture.md)):

- **`superdev-core`** — all domain logic: blueprint model, component
  definitions, state observation, diffing, planning. Pure where possible;
  filesystem and process side-effects behind narrow traits.
- **`superdev` (binary)** — clap parsing and wiring. This sub-project adds
  `init`, `status`, `sync`, `update`.
- **The blueprint** — superdev's opinion of a managed repo, compiled into
  the binary: a set of components plus a registry of default versions tested
  together. The binary's version is the blueprint version.

## Files in a managed repo

- `.superdev/config.toml` — the manifest: blueprint version, enabled
  capabilities, providers, version pins. Committed, human-editable. Source
  of truth for what the repo wants.
- `.superdev/lock.toml` — what superdev last applied: per-component versions
  and content hashes of superdev-owned files. Committed. How a sync warns
  before overwriting a deliberate user edit.
- `.superdev/cache/` — gitignored machine state: backups, and (sub-project
  2) the search index and embedding store.

## Capabilities and providers

The manifest is organised by capability; the tool behind each is a swappable
provider. CLI flags name capabilities, never tools (`--no-code-index`, not
`--no-codegraph`).

```toml
[knowledge]   # provider = "aokf" (native)
[code-index]  # provider = "codegraph"
[workflows]   # provider = "superpowers"
[frontend]    # provider = "frontend-design"
[skills]      # provider = "superdev-plugin" (slot; filled by sub-project 3)
```

In core, providers implement one trait and register under a capability;
swapping codegraph for another indexer later adds a provider without
touching the user-facing surface.

# Component model

Each provider implements:

- `observe(repo) -> State` — read-only: what is installed now, at what
  version.
- `desired(manifest) -> State` — what config.toml asks for.
- `diff(current, desired) -> Vec<Action>` — pure; this is what `status`
  prints.
- `apply(actions)` — the only place side-effects happen (file writes,
  `mise install`, `claude plugin …`, `codegraph init`).

Every verb composes these steps, so dry-run and drift-reporting are
observe+diff without apply.

## File ownership

Three rules for files a component touches:

1. **superdev-owned** (e.g. the embedded AOKF spec written to
   `.agents/aokf/SPEC.md`): hashed into lock.toml; rewritten freely on
   `sync`, with a warning listing what will be overwritten when the hash
   shows a user edit.
2. **User-owned scaffolds** (e.g. `AGENTS.md`, `knowledge/*.md` content):
   created by `init` if missing, never touched again. Never drift.
3. **Shared files** (`.mise.toml`): targeted TOML edits preserving user
   content, formatting and comments; only superdev-managed keys are hashed.

External state (installed Claude Code plugins, codegraph's index) is
observed live via the owning tool, not hashed; the lock records only the
versions superdev installed.

# Verbs

**`superdev init`** — refuses outside a git repo or when `.superdev/`
exists (points at `sync`). Writes config.toml with every capability enabled
at registry versions (`--no-<capability>` to disable), prints the full plan,
applies in dependency order: mise pins → `mise trust` + `mise install` → plugins
(marketplace add + install) → `mise exec -- codegraph init` → AOKF skeleton + AGENTS.md
scaffold. Writes lock.toml and gitignores `.superdev/cache/`. As shipped it
prints no commit hint.

**`superdev status`** — observe + diff, per-component report. As shipped each
component prints `ok` or `N change(s)` with the pending actions listed
beneath, and a pin behind the registry adds its own line. No writes; exit 1 on
any finding, so CI can gate on it.

**`superdev sync`** — the same plan, then apply. `--dry-run` prints without
applying.

**`superdev update [<capability>[@<version>]]`** — rewrites pins in
config.toml to this binary's registry defaults (or the named version), then
syncs. Updating superdev itself is out of scope (cargo/npm do that).

# Testing

- **Unit (bulk of the ≥90% gate):** `diff` and parsing are pure — feed
  observed/desired states, assert action lists.
- **Integration:** components against temp-dir fake repos; external
  commands behind a process-runner trait, faked to record invocations and
  script outcomes (including failures mid-apply, to exercise rollback).
- **CLI end-to-end (assert_cmd):** `init` in a temp git repo with fakes on
  PATH → `status` clean → mutate a superdev-owned file → `status` exits 1 →
  `sync` repairs.
- **Smoke (real tools):** a devcontainer-only script running real `init`
  against a scratch repo; manual, pre-release, not in CI (network + claude
  auth).

# Out of scope

The MCP server and `aokf` verbs (sub-project 2); skill pack contents
(sub-project 3); knowledge upkeep and blueprint migrations (sub-project 4);
support for agent runtimes other than Claude Code; updating the superdev
binary itself.

# Test plan: cli core & blueprint engine

## Scope

- The engine, the components and the verbs, as described above.
- Out: everything the sections above place out of scope.

## Risks driving this plan

1. Recorded after the fact. This plan was written when the spec was
   conformed to its contract, not when the feature was built, so it names
   the risks the tests actually cover rather than the ones weighed at the
   time.

## Test cases

### Automated

| # | Case | Type | Inputs / setup | Expected result |
|---|------|------|----------------|-----------------|
| 1 | The diff is pure | unit | observed and desired state | the expected action list |
| 2 | Components against a temp-dir repo | integration | a fake repo and a fake runner | the planned actions, with no process spawned |
| 3 | The verbs end to end | end-to-end | the real binary in a scratch repo | the documented exit codes |

### Manual verification

1. None recorded. The feature shipped under the automated cases above; no
   manual step was written down at the time, and inventing one now would
   claim a check nobody made.

## Exit criteria

- The automated cases above pass.
- `superdev validate` reports no error for this document.
