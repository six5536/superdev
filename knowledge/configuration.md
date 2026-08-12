---
type: Reference
id: configuration
title: Configuration & Environments
description: The .superdev directory — the config.toml manifest, the lock file and the gitignored cache — plus the embeddings opt-in, .mcp.json and the user-level model cache.
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

One table per enabled capability, keyed by the capability name — an absent
table means disabled, which is what `init --no-<capability>` produces. An
unknown capability name is rejected. `version` is omitted where the source
manages versions itself. A capability pinned below the registry default reads
as behind; a pin above it is deliberate and left alone.

# `lock.toml`

What superdev last applied. Committed, never hand-edited. It records the
provider and version applied per capability, plus a sha256 of every file
superdev owns (mise pins under the key `.mise.toml:<tool>`). Drift is found by
comparing a file against the content the blueprint wants, not against the lock;
the hashes are what lets an apply tell that the file it just overwrote had been
edited by hand, and say so (after backing it up).

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
- The local embedding model lives in the *user* cache, not the repo:
  `$XDG_CACHE_HOME` (else `%LOCALAPPDATA%`, else `~/.cache`) +
  `/superdev/models/<model>/<revision>/`. Revision-scoped, so a pin bump
  downloads afresh instead of overwriting. One ~130 MB download serves every
  repo on the machine; see [technology-stack](technology-stack.md) for the pin
  itself.

The capability set is in [architecture](architecture.md); the file-ownership
rules that decide what gets hashed are in [glossary](glossary.md).
