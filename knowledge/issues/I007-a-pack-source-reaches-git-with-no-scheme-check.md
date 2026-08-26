---
type: Issue
id: issue-007-a-pack-source-reaches-git-with-no-scheme-check
title: A pack source reaches git unvalidated, so a repo's manifest can run a command through a transport helper
description: "The source string is handed to git clone as the URL with no scheme allowlist and no end-of-options separator, so a manifest naming a command-running transport helper executes it — confirmed on a git that permits the transport, and superdev contributes no defence of its own."
status: draft
tags: [needs-triage]
links:
  - rel: references
    to: spec-content-packs
  - rel: relates-to
    to: security-requirements
---

# Bug: a pack source reaches git with no scheme check

## Summary

Against [S014](../specs/S014-content-packs-design.md).

`[[packs]].source` is passed to `git clone` as the URL with no validation of
its scheme and no `--` end-of-options separator. Git's `ext::` transport takes
a **command** as its connection, so a manifest naming one makes `superdev sync`
run it — before any digest is verified, since verification checks the result of
a fetch that has already happened.

The manifest is a committed file, so this is reachable by cloning a branch and
running `sync` in it: a fork, a contributor's PR, any untrusted project. It
narrows the local-by-default guarantee in
[security-requirements](../security-requirements.md) further than that concept
describes.

Not exploitable on a stock git — 2.51.1 refuses `ext` by default, verified with
raw `git clone` and no config or environment set. It is exploitable for anyone
who has enabled the transport, which people do for custom transports.
**superdev itself contributes no defence at any point**, which is the defect:
the trust model says naming a source is the user's trust decision, and this
makes merely *resolving* someone else's source execute their code.

## Environment

- Version/commit: 0.2.0 / P003 complete (`e1ac431`)
- Platform: all; git 2.51.1 used for the reproduction

## Steps to reproduce

1. `printf '[protocol "ext"]\n\tallow = always\n' > /tmp/gitcfg` — the
   non-default configuration this needs.
2. In a scratch repo, `superdev init`, then append:
   ```toml
   [[packs]]
   source = "ext::touch /tmp/PROOF"
   rev = "main"
   ```
3. `GIT_CONFIG_GLOBAL=/tmp/gitcfg superdev sync`

## Expected behaviour

A source whose scheme superdev does not support is refused before anything is
spawned, naming the source.

## Actual behaviour

`/tmp/PROOF` is created — the command ran. `sync` then reports the clone
failure, having already executed it. On a stock git the run instead fails with
`fatal: transport 'ext' not allowed`, so the defence is entirely git's.

## Root cause (if known)

`is_git` at `crates/lib/superdev-core/src/pack/source.rs:258` accepts any
`word:...` form as a git source, with no scheme allowlist. `clone_url`
(`source.rs:149`) returns anything that is not the `github`/`gitlab` shorthand
verbatim. `fetch` (`crates/lib/superdev-core/src/pack/fetch.rs:116`) pushes it
as the clone operand with no `--` before it.

## Proposed fix / workaround

- Fix, in order of value:
  1. Allowlist the schemes superdev supports — `https://`, `ssh://`, `git://`,
     `file://`, the `github:`/`gitlab:` shorthands and the `user@host:path` scp
     form — and refuse the rest, naming the source.
  2. Refuse a source or rev that begins with `-`, and put `--` before the
     operands in every git invocation (`fetch.rs`, and `pin.rs`'s `ls-remote`
     for consistency). Without it a value like `--upload-pack=id:x`, which
     `is_git` accepts, is read by git as an option.
  3. Add `-c protocol.ext.allow=never` to the `verbatim()` overrides, so the
     defence holds whatever the user's config says.
- Workaround: do not run `superdev sync` in a repository whose manifest you
  have not read.

## Regression risk

`pack/source.rs`'s source recognition and `pack/fetch.rs`'s argument vectors;
an allowlist risks refusing a spelling someone legitimately uses, so the
existing `only_the_shorthand_is_expanded_for_git` cases should all still pass.
A test would assert an `ext::` source is refused before any command runs.
