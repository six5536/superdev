---
type: Convention
id: error-handling
title: Error Handling & Logging
description: Exit codes, the broken-pipe rule, the validation hook's blocking exit 2, why MCP tool failures never end the process, and how a failed apply reports what it could not undo.
status: stable
resource: /crates/app/superdev/src/main.rs
---

# Exit codes

- `0` — success.
- `1` — a check found something, not an error: `status` found work to do
  (drift, a missing component, an orphaned lock entry to sweep, or a pin behind
  the registry), or `aokf validate` found errors in the bundle. CI gates on
  both.
- `2` — usage error (clap), a hard failure, or an I/O failure, rendered as
  `error: <message>` on stderr. A failed `sync` or `init` exits `2`, as does
  `mcp aokf` when it cannot start. A provider id the registry does not know —
  in the manifest, or behind `update --provider` — exits `2`
  with `<capability> provider must be one of: <ids>`. The validation hook uses
  `2` for its own purpose — see below. `status` reaches `2` in one case: an
  orphaned lock entry it cannot read, because the engine never guesses about
  content it cannot see.

A closed stdout pipe is the one I/O failure that is not an error: a reader
that stops early (`| head`, a pager quit) ends the run at `0`, silently.

`aokf validate`'s three codes are the Python validator's, kept identical so
that everything gating on the old script's exit code — the hook,
`npm run check:aokf`, CI — gates the same way on the binary. See
[development-commands](development-commands.md).

# The validation hook

`aokf hook validate` speaks Claude Code's hook protocol instead. It exits `0`
whenever Claude Code should let the edit through: the payload names no file, or
names one outside `knowledge/`, or names one the bundle still validates
against. Otherwise the findings go to stderr and it exits `2`, which Claude
Code hands back to the agent as a blocking error. A payload it cannot read or
parse is a loud `2` too — skipping silently would silently stop validating the
bundle.

A missing `superdev` on PATH never reaches any of that: the hook command fails
to start, which Claude Code reports as a failed command rather than a block.
Softer than exit `2`, and a machine without the binary cannot run any superdev
verb anyway.

# MCP tools never exit

Inside `mcp aokf` a failure is a tool error payload, and the process keeps
serving. An unknown id, a file the parser choked on, an embedding model that
will not load — each answers the one call that hit it and leaves the session
alive, because the client's next question may well be answerable. The startup
checks are the exception: they run before any client is listening. Search with
no model degrades to lexical-only with a warning in the response, never an
error.

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
