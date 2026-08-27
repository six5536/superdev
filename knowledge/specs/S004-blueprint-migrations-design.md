---
type: Spec
id: spec-blueprint-migrations
title: Blueprint Migrations
description: Design for evolving a managed repo — components declare what they own, the lock's leftovers are pruned or released, and CLAUDE.md imports AGENTS.md so Claude Code reads it at all.
status: stable
links:
  - rel: relates-to
    to: spec-cli-core-blueprint-engine
    note: Sub-project 4; completes the engine's promise that a repo can follow the blueprint over time.
  - rel: relates-to
    to: spec-skill-pack
    note: Reuses the custom-skill rule — superdev never takes back what it did not write.
---

# Goal

Let a managed repo follow the blueprint as the blueprint changes. Today
superdev only ever adds: a file it stops shipping stays forever, a renamed
file leaves its old copy behind, and disabling a capability leaves its pins
and registrations in place. Stale files here are not clutter — they are
instructions an agent still reads.

This sub-project also fixes a defect the goodbye-tinnitus trial exposed:
Claude Code does not read `AGENTS.md`, so everything superdev writes into it
is invisible unless `CLAUDE.md` imports it.

# Sub-project decomposition

The remaining work after the [blueprint
engine](S001-cli-core-blueprint-engine-design.md), the [AOKF MCP
server](S002-aokf-mcp-server-design.md) and the [skill
pack](S003-skill-pack-design.md) splits three ways:

4. **Blueprint migrations** — this spec: pruning what the blueprint dropped,
   and the entry-point fix.
5. **Structured AOKF update over MCP** — the write side postponed out of
   sub-project 2: tools that create and edit concepts with the write classes
   enforced in code.
6. **Knowledge upkeep** — the knowledge-capture skill, lapsed-verification
   reporting, keeping the canonical knowledge true as the code moves.

Both 5 and 6 are out of scope here and must not creep in.

# Migration model: derived, not scripted

There are no migration scripts and no version-keyed steps. What superdev put
in a repo is recorded in the lock; what the blueprint wants now is declared
by the components. Anything in the first set and not the second is an
orphan. One rule covers dropped files, renames (the old path simply stops
being claimed), disabled capabilities, and swapped providers.

Version-keyed migrations were rejected: every routine file drop would need
hand-written machinery, and the diff already knows. A transformation a diff
cannot infer — moving a user's prose from one file to another — is not
something superdev should attempt unattended anyway.

## Components declare what they own

`Component` gains one method beside `plan`:

```rust
/// Everything this component owns in a managed repo, whether or not it
/// needs changing. The orphan pass subtracts these from the lock.
fn owned(&self, ctx: &Ctx<'_>) -> Vec<Claim>;
```

`Claim` is the typed form of a lock entry: `File(path)`, `MisePin(tool)`,
`JsonKey { path, pointer }`. Components answer from the constants they
already plan from — the canonical knowledge provider its owned files plus the
`.mcp.json` server key, the skill pack its non-`custom` skills plus the
settings hook entry, codegraph and superpowers their pins, frontend nothing.

This cannot be derived from `plan` output: a converged file plans no action,
so an in-sync repo would look entirely orphaned.

## The orphan pass

Repo-level, planned alongside the `.gitignore` entry, and planned **last**
so its actions run after every component write — a rename whose write fails
rolls back before anything is deleted.

For each lock entry no live claim covers:

- **Unmodified** (content still hashes to the locked value) — superdev's own
  residue. Removed, backed up to `.superdev/cache/backup/<stamp>/` like any
  overwrite, journalled so a later failure restores it.
- **Modified** — the user's work under superdev's old name. Left exactly as
  it is, dropped from the lock, and reported once. This is the
  [skill pack](S003-skill-pack-design.md)'s custom rule again:
  superdev takes back only what it wrote.
- **Already gone** — leaves the lock silently.
- **Unreadable, or now a directory** — fails loudly, as everywhere else the
  engine refuses to guess about content it cannot read.

Three actions carry it: `RemoveFile`, `RemoveMisePin`, `RemoveJsonKey`. All
journal as a file restore, because removing a pin or a key is a file
rewrite. Removing the last key from a JSON object leaves the empty parent
(`{"mcpServers": {}}`); guessing which empty containers a user wants gone is
worse than the residue.

Whether the lock keeps one flat map with shape-classified keys or splits
into typed tables is the implementation's call. Either way a lock written by
0.1.0 must still read correctly, with a test that says so.

# The entry point

Claude Code reads `CLAUDE.md`, not `AGENTS.md`. In goodbye-tinnitus, `init`
wrote a careful `AGENTS.md` that Claude Code will never load, because that
repo's `CLAUDE.md` does not reference it. This repo works only because its
`CLAUDE.md` happens to contain `@AGENTS.md`.

`sync` ensures `CLAUDE.md` contains the line `@AGENTS.md` — appended to
whatever is already there, or created as a one-line file. `AGENTS.md` stays
canonical and tool-neutral, which is Anthropic's own documented guidance for
repos serving more than one agent tool; imports nest four deep, enough for
`AGENTS.md`'s own `@.agents/*.md` chain.

The line is an `EnsureLine` owned by the knowledge component, so it behaves
like the `.gitignore` lines: added when missing, never rewritten, never
hashed into the lock, and therefore never an orphan. Deleting it means the
next `sync` restores it — the bargain `.gitignore` already makes.

# The blueprint version

`blueprint` in the manifest becomes *the version last applied*: `sync`
stamps the binary's version on a successful run, rewriting `config.toml`
only when the value changes. `status` prints the difference
(`blueprint 0.1.0, binary 0.4.0 — sync will update it`) without failing on
it alone; real work still decides the exit code, so a binary upgrade that
changes nothing keeps CI green.

# Exit codes and reports

- Removals are planned actions: `status` exits 1 while any remain.
- Released orphans and the blueprint-version line are reports. They never
  affect the exit code — a settled state is not drift, the same ruling
  `[skills] custom` already has.
- `update` inherits all of this, since it funnels into `sync`.

# Testing

- Per component: `owned()` lists exactly what it plans, and the skill pack
  omits `custom` skills.
- Orphan computation: extra lock entries, a modified file, a disabled
  capability orphaning its whole set, an already-deleted path.
- Engine: each removal action's backup, journal entry and unwind restore.
- Lock back-compatibility: a 0.1.0-format lock reads correctly.
- End to end: `status` exits 1 on an unclaimed lock path and `sync` removes
  it; a modified orphan survives on disk and is reported; a repo with an
  existing `CLAUDE.md` gets the import appended with its content intact, and
  a repo without one gets the one-line file.

Dogfooding is quiet here: this repo's `CLAUDE.md` already imports
`AGENTS.md` and every lock entry is still claimed, so it exercises the no-op
path. The real proof is a fresh goodbye-tinnitus adoption, where the import
line decides whether Claude Code loads superdev's rules at all.
