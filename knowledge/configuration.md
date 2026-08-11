---
type: Reference
id: configuration
title: Configuration & Environments
description: The .superdev directory — the config.toml manifest, the lock file, and the gitignored cache.
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
`backup/<timestamp>/` today, and the search index later. Deleting it is safe.

The capability set is in [architecture](architecture.md); the file-ownership
rules that decide what gets hashed are in [glossary](glossary.md).
