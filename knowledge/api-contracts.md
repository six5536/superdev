---
type: Reference
id: api-contracts
title: API Contracts
description: The CLI surface — the four manage verbs, the packaging plumbing, and the stability promises.
status: stable
resource: /crates/app/superdev/src/main.rs
---

# CLI surface

```
superdev                     print help, exit 0
superdev init                set this repo up; --no-knowledge, --no-code-index,
                             --no-workflows, --no-frontend disable a capability
superdev status              report drift; exit 1 when there is work to do
superdev sync                re-apply the blueprint; --dry-run prints the plan only
superdev update [TARGET]     move pins to this binary's defaults, then sync;
                             TARGET is `<capability>[@<version>]`
superdev completions <SHELL> write a completion script to stdout
                             (bash | zsh | fish | powershell | elvish)
superdev man                 (hidden; roff to stdout, for packaging)
-V, --version                print `superdev x.y.z` and exit
```

Every verb acts on the current directory.

- **`init`** refuses a directory that is not a git repo, and refuses a re-run
  once `.superdev/` exists (it points at `sync`). It writes the manifest, then
  applies the whole blueprint and the `.gitignore` lines.
- **`status`** never writes. It exits `1` on any drift, missing component, or
  pin behind this binary's registry, so CI can gate on it.
- **`sync`** refuses to run while `workflows` is pinned anywhere other than the
  registry default, and says to run `superdev update`. The pinned tarball
  checksum is the only provenance superdev has for that plugin. On a fresh
  clone it runs `mise trust` then `mise install` before any provider command,
  because the committed pins need no edit yet name tools this machine has
  never installed — and mise will not install from a config this machine has
  never trusted.
- **`update`** rejects an explicit `workflows@<version>` for the same reason.
  Every other capability takes an explicit version.

A usage error (unknown flag or subcommand) exits `2` — the npm launcher's
smoke test relies on that code. `completions` and `man` render into a buffer
before writing, because `clap_complete` panics rather than returning an error
when a write fails. Exit codes are in [error-handling](error-handling.md); the
manifest the verbs read is in [configuration](configuration.md).

# Stability

Unreleased. Everything above may change without notice; `superdev-core`'s Rust
API is not stable.
