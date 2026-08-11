---
type: Convention
id: error-handling
title: Error Handling & Logging
description: Exit codes, the broken-pipe rule, and how a failed apply reports what it could not undo.
status: stable
resource: /crates/app/superdev/src/main.rs
---

# Exit codes

- `0` — success.
- `1` — `status` found work to do: drift, a missing component, or a pin behind
  the registry. Not an error; CI gates on it.
- `2` — usage error (clap), a hard failure, or an I/O failure, rendered as
  `error: <message>` on stderr. A failed `sync` or `init` exits `2`.

A closed stdout pipe is the one I/O failure that is not an error: a reader
that stops early (`| head`, a pager quit) ends the run at `0`, silently.

# Failed applies

The engine journals the inverse of every side effect as it makes it: files
are backed up to `.superdev/cache/backup/<timestamp>/` before being
overwritten, and a plugin install pairs with its uninstall. The first failure
stops the run and unwinds the journal in reverse, best-effort.

The report then names each undone step as `reverted:` and everything else as
`NOT reverted:` — a command with no inverse, or an undo that itself failed.
Nothing is left behind unreported: a failed `init` also keeps
`.superdev/config.toml`, written before the apply and named in the output,
because it is what the retry resumes from. `sync` is the recovery path for
whatever the unwind could not restore.

External command failures carry the exact command line and verbatim stderr. A
missing `claude` is not fatal: plugin steps are optional and report as skipped
with the reason. See [architectural-rules](architectural-rules.md) for why the
engine is the only place any of this can happen.
