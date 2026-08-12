---
type: Reference
id: configuration
title: Configuration & Environments
description: The .superdev directory — the config.toml manifest, the lock file and the gitignored cache — plus the embeddings opt-in, the skills custom list, the .mcp.json and .claude/settings.json merges, and the user-level model cache.
status: stable
resource: /crates/lib/superdev-core/src/manifest.rs
---

Everything superdev keeps in a managed repo lives under `.superdev/`.

# `config.toml` — the manifest

What the repo wants. Committed and hand-editable; superdev only rewrites it on
`update`. `init` writes it from the registry defaults.

```toml
blueprint = "0.1.0"      # the superdev version that wrote this

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

[workflows]
provider = "superpowers"
version = "6.2.0"
```

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
name that is not one of the five shipped skills fails the plan.

`init` seeds that list: a repo that already has a skill under a pack name,
with content of its own, keeps it — the name goes into `custom` and the
adoption reports it. Content byte-identical to the shipped skill is superdev's
own text and is left managed.

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

# `cache/`

Machine state, gitignored by `init`: backups of overwritten files under
`backup/<timestamp>/`, and the search index under `aokf-index/` (tantivy, the
section vectors, and a manifest of per-file hashes, schema version and model
id). Deleting it is safe — the next tool call rebuilds it.

# Outside the repo

- `.mcp.json` at the repo root registers the MCP server under
  `mcpServers.superdev-aokf`. The file is shared with the user's own servers,
  so superdev manages and hashes that one key and leaves the rest alone, the
  same rule it applies to `.mise.toml`. A managed repo gets
  `superdev mcp aokf`; this repo, which has no installed binary, gets
  `cargo run --quiet -- mcp aokf`.
- `.claude/settings.json` carries one managed `hooks.PostToolUse` element, the
  array-element analogue of the `.mcp.json` key merge: superdev finds its own
  element by the command string `superdev aokf hook validate`, adds or updates
  it, and leaves the user's hooks alone. Both files are re-serialised whole on
  write, so key order is not preserved; the lock hashes the merged value, not
  the file, so a reformat is not drift.
- `.claude/skills/<name>/SKILL.md` holds the five shipped skills as owned
  files. A `PROJECT.md` beside one extends it — superdev never writes, hashes
  or reads that file, so a project layer survives every sync.
- The local embedding model lives in the *user* cache, not the repo:
  `$XDG_CACHE_HOME` (else `%LOCALAPPDATA%`, else `~/.cache`) +
  `/superdev/models/<model>/<revision>/`. Revision-scoped, so a pin bump
  downloads afresh instead of overwriting. One ~130 MB download serves every
  repo on the machine; see [technology-stack](technology-stack.md) for the pin
  itself.

The capability set is in [architecture](architecture.md); the file-ownership
rules that decide what gets hashed are in [glossary](glossary.md).
