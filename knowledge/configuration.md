---
type: Reference
id: configuration
title: Configuration & Environments
description: The .superdev directory — the config.toml manifest, the lock file, and the gitignored cache — plus the embeddings opt-in, the custom lists, the removed-workflows guided error, the .mcp.json and .claude/settings.json merges, and the user-level model cache.
status: stable
resource: /crates/lib/superdev-core/src/manifest.rs
---

Everything superdev keeps in a managed repo lives under `.superdev/`.

# `config.toml` — the manifest

What the repo wants. Committed and hand-editable; superdev rewrites it on
`update` and to stamp `blueprint`. `init` writes it from the registry
defaults.

```toml
blueprint = "0.1.0"      # the superdev version last applied

[code-index]
provider = "codegraph"
version = "1.5.0"

[frontend]
provider = "frontend-design"

[knowledge]
provider = "aokf"

[skills]
provider = "superdev-skills"
version = "0.1.0"
```

`blueprint` is the version last applied, not the version that wrote the file.
A successful `sync` stamps this binary's version, rewriting `config.toml` only
when the value changes; `--dry-run` never stamps. `status` prints
`blueprint <a>, binary <b> — sync will update it` and leaves the exit code
alone, so a binary upgrade that changes nothing keeps CI green.

`provider` names the implementation filling the slot. Every capability
currently has one provider; an id the registry does not carry fails with
`<capability> provider must be one of: …`. A manifest still naming the
removed `workflows` capability fails at load with a guided error telling the
user to delete the table (moving any custom names to `[knowledge]`) — the
skill set now ships with the knowledge capability, and superpowers users can
`claude plugin install superpowers` by hand. The error never rewrites
`config.toml`; the manifest is the user's file
([spec](specs/S009-knowledge-carried-skills-design.md)).

An optional sub-table opts the knowledge bundle out of the local embedding
model and onto an API:

```toml
[knowledge.embeddings]
provider = "openai"              # the only provider implemented
model = "text-embedding-3-small"
```

The API key comes from `OPENAI_API_KEY` and never from the file. The recorded
model id is part of the index manifest, so switching provider rebuilds the
index by itself. Absent the table, embedding is local and offline after the
first download.

`[skills]` takes an optional `custom` list naming skills released from
management:

```toml
[skills]
provider = "superdev-skills"
version = "0.1.0"
custom = ["humanise"]
```

A released skill keeps whatever content it has as a starting point, drops out
of the plan and out of the lock, and `status` prints it as unmanaged rather
than drifted. Delete the name to get stock content back on the next sync. A
name the pack does not ship reports `skills: custom names
unknown skill '<name>' — no effect` and changes nothing, so a pack that drops
a skill does not break a repo that had marked it custom.

`init` seeds that list: a repo that already has a skill under a pack name,
with content of its own, keeps it — the name goes into `custom` and the
adoption reports it. Content byte-identical to the shipped skill is superdev's
own text and is left managed.

`[knowledge] custom` releases an aokf-carried skill the same way — the whole
skill directory, companions included — with the same `init` adoption and the
same `<capability>: custom names unknown skill '<name>' — no effect` line
for a name the capability does not ship. The lists are name-guarded: a name
in one capability's list never releases another capability's file, even
though both write into `.claude/skills/`.

A `[template]` table records the project template `init` seeded the repo
from, with the substituted token values:

```toml
[template]
name = "rust-npm"
project-name = "My Tool"
project-slug = "my-tool"
```

Provenance, not management: template files are scaffolds, so nothing here is
hashed and no verb re-plans them. Absent when init chose no template.

One table per enabled capability, keyed by the capability name — an absent
table means disabled, which is what `init --no-<capability>` produces. An
unknown capability name is rejected. `version` is omitted where the source
manages versions itself. A capability pinned below the registry default reads
as behind; a pin above it is deliberate and left alone.

# `lock.toml`

What superdev last applied. Committed, never hand-edited. It records the
provider and version applied per capability, plus a sha256 of every file
superdev owns. Entries superdev merges into a shared file are hashed under
`<file>:<pointer>` instead — `.mise.toml:<tool>` for a pin,
`.claude/settings.json:hooks.PostToolUse[<marker>]` for the hook. Drift is
found by comparing a file against the content the blueprint wants, not against
the lock; the hashes are what lets an apply tell that the file it just
overwrote had been edited by hand, and say so (after backing it up).

A legacy `owners` table may remain from binaries that materialised skills
from provider checkouts. Nothing writes or reads it any more: every shipped
file's name comes from the binary, so claims need no attribution. The first
sync clears the whole table — a per-file retirement would strand entries on
files that never need rewriting — and an empty table is not serialised.

An entry no component claims any more is an orphan, and `sync` prunes it.
Content that still hashes to the locked value is superdev's own residue: it is
removed, backed up like any overwrite, and restored if a later step fails.
Content the user changed is left exactly where it is, dropped from the lock,
and reported once as `orphan: <key> changed since superdev wrote it — left in
place, released from the lock`. An entry whose target is already gone leaves
the lock silently; one that cannot be read fails the run. Pins and merged JSON
keys are pruned the same way, and a disabled capability's `components` record
goes with its files.

# `cache/`

Machine state, gitignored by `init`: backups of overwritten files under
`backup/<timestamp>/`, and the search index under `aokf-index/` (tantivy, the
section vectors, and a manifest of per-file hashes, schema version and model
id). Deleting it is safe — the next tool call rebuilds it.

# Outside the repo

- `.mcp.json` at the repo root registers the MCP servers: the knowledge
  bundle under `mcpServers.superdev-aokf`, and — with code-index enabled —
  codegraph's under `mcpServers.codegraph`, launched through `mise exec`
  because the pinned binary is on no PATH the client can see. The file is
  shared with the user's own servers, so superdev manages and hashes its
  own keys and leaves the rest alone, the same rule it applies to
  `.mise.toml`. A managed repo gets `superdev mcp aokf`; in this repo the
  dev shim makes that resolve to `cargo run` against the working tree.
- AGENTS.md carries one ensured line, `@.agents/superdev.md`, and is
  otherwise the user's. The aggregator it imports and the per-capability
  instruction files beside it (`.agents/aokf.md`, `.agents/codegraph.md`)
  are owned files; the general rules (`.agents/coding.md`,
  `.agents/prose.md`) are write-once scaffolds, the user's to adapt.
- `.claude/settings.json` carries one managed `hooks.PostToolUse` element,
  owned by the knowledge capability (the hook validates the bundle, so it
  exists exactly where a bundle does): superdev finds its own element by the
  command string `superdev aokf hook validate`, adds or updates it, and
  leaves the user's hooks alone. Both files are re-serialised whole on
  write, so key order is not preserved; the lock hashes the merged value, not
  the file, so a reformat is not drift.
- `.claude/skills/` holds the pack's three skills and the knowledge
  capability's 25 carried skill directories as owned files, plus the MIT
  notice for the derived set. A `PROJECT.md` beside a skill extends it —
  superdev never writes, hashes or reads that file, so a project layer
  survives every sync.
- The local embedding model lives in the *user* cache, not the repo:
  `$XDG_CACHE_HOME` (else `%LOCALAPPDATA%`, else `~/.cache`) +
  `/superdev/models/<model>/<revision>/`. Revision-scoped, so a pin bump
  downloads afresh instead of overwriting. One ~130 MB download serves every
  repo on the machine; see [technology-stack](technology-stack.md) for the pin
  itself.

The capability set is in [architecture](architecture.md); the file-ownership
rules that decide what gets hashed are in [glossary](glossary.md).
