---
type: ConfigContract
id: contract-004-config-superdev
title: Configuration Contract
description: What a managed repo supplies to superdev — the manifest keys, the three environment variables, and which source defines what.
lifecycle: active
resource: /crates/lib/superdev-core/src/manifest.rs
---

# Config contract: superdev

What a managed repo supplies to superdev: the manifest keys, the three
environment variables, and which source defines what.

## Settings

Environment, read on every run:

| Name | Type | Default | Meaning |
|------|------|---------|---------|
| `OPENAI_API_KEY` | string | none — required only with `[knowledge.embeddings]` | the key for the embedding API |
| `CLAUDE_PROJECT_DIR` | path | the working directory | the repository `hook validate` resolves against; Claude Code sets it |
| `XDG_CACHE_HOME` | path | `%LOCALAPPDATA%`, else `~/.cache` | the parent of the user-level model cache |

`.superdev/config.toml`, the manifest, hand-edited and committed:

| Name | Type | Default | Meaning |
|------|------|---------|---------|
| `blueprint` | version | written by `init` | the superdev version last applied |
| `[[packs]]` | array of tables | the pack embedded in the binary | the content packs to layer, in layer order |
| `<capability>.provider` | string | the registry default | which implementation fills the slot |
| `<capability>.version` | string | the registry default | the pin, for a capability that takes one |
| `skills.custom`, `knowledge.custom` | list of strings | empty | skills released from management, by name |
| `[knowledge.embeddings]` | table | absent — embedding is local and offline | opts the index onto an API |
| `[template]` | table | absent | the project template `init` seeded the repo from, and its token values |

The behaviour behind each key is in [configuration][sokf:configuration];
what is promised here is the key, its shape and its default.

## Sources and precedence

- A command-line flag, where the command defines one. `validate` takes
  `--knowledge <DIR>` and `--repo-root <DIR>`; the flags themselves are
  defined by [contract-002-cli-superdev][sokf:contract-002-cli-superdev].
- The environment, for the three variables above and nothing else.
- `.superdev/config.toml`, for everything else.
- The built-in defaults: the registry default per capability, and the pack
  compiled into the binary.

The four sources are disjoint, which is the promise that matters: a
setting MUST NOT be read from two of them, so nothing silently overrides
anything. A value's source is decided by which of the four defines it,
not by a precedence order. The one deliberate exception runs the other
way — the embedding API key MUST be read from the environment and MUST
NOT be accepted from the manifest, so it cannot be committed.

Both the manifest and the environment are read afresh on every run. No value
is cached between commands.

## File

```toml
blueprint = "0.2.0"                  # the superdev version last applied

[[packs]]                            # absent = the pack embedded in the binary
source = "github:six5536/superdev"   # git: a rev is required
rev    = "assets-v1.4.0"

[[packs]]
source = "./packs/acme"              # a path on this machine: no rev

[code-index]
provider = "codegraph"
version  = "1.5.0"

[knowledge]
custom = ["humanise"]                # skills released from management

[knowledge.embeddings]               # absent = local, offline embedding
provider = "openai"
model    = "text-embedding-3-small"

[skills]
provider = "superdev-skills"
version  = "0.2.0"
```

An absent capability table means the capability is disabled. An absent
`[[packs]]` array means the embedded pack, which is why `init` writes the
default entry rather than leaving it out: both resolve alike, but the written
pin is the one a reader can see and edit. A `provider` the registry does not
carry fails with `<capability> provider must be one of: …`, and a manifest
still naming the removed `workflows` capability fails at load with the edit to
make. Neither error rewrites the file: the manifest is the user's.

## Secrets

`OPENAI_API_KEY` is the only credential superdev reads, and only when
`[knowledge.embeddings]` opts the index onto an API. It MUST be read from
the environment and MUST NOT be read from the manifest, so it cannot
reach a commit.

A git call superdev makes MUST NOT prompt for credentials: a pack source
needing them fails rather than waiting for someone to type. A credential
MUST NOT enter superdev by any path other than that one variable.

## Stability

Unreleased. Key names, defaults and the variables above MAY change
without notice. What holds even so: a manifest superdev cannot
understand MUST fail at load naming the edit to make, and MUST NOT be
rewritten.

<!-- sokf:links -->
[sokf:configuration]: /knowledge/configuration.md
[sokf:contract-002-cli-superdev]: /knowledge/contracts/public/active/contract-002-cli-superdev.md
