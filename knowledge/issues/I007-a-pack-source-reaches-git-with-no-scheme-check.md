---
type: Issue
id: issue-007-a-pack-source-reaches-git-with-no-scheme-check
title: A pack source's scheme is unchecked, so the base pack can be fetched over a transport anyone on-path can answer
description: "The command-running half is closed; what remains is that superdev allowlists no scheme, so git:// and http:// normalise onto the default identity and a cloned manifest can have the base pack fetched over an unauthenticated transport."
status: draft
tags: [needs-triage]
links:
  - rel: references
    to: spec-content-packs
  - rel: relates-to
    to: security-requirements
---

# Bug: a pack source's scheme is unchecked

## Resolved in part

P003 slice 15 closed the command-execution half. Every git call is built by
one function that puts `-c protocol.ext.allow=never` in front, so an `ext::`
URL runs nothing whatever the user's git config says; a source or rev
beginning with `-` is refused at parse; and `--` precedes every operand. Two
regression tests hold it, one per verb — `update`'s query was a second way in
that `sync` could not reach.

What remains is below, and it needs the interface decision this issue was
always going to need.

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

1. In a scratch repo, `superdev init`, then point the pack entry at the
   default repository over an unauthenticated transport:
   ```toml
   [[packs]]
   source = "git://github.com/six5536/superdev"
   rev = "assets-v0.2.0"
   ```
2. `superdev sync`

`git_identity` discards the scheme, so this keys as the default pack — which
means it *replaces* the embedded content rather than layering over it, and it
is fetched over a transport with no authentication and no integrity.

## Expected behaviour

A source whose scheme superdev does not support is refused before anything is
spawned, naming the source.

## Actual behaviour

The clone goes out over `git://`. Nothing refuses it, and because identity
ignores the scheme the content replaces superdev's own. The same holds for
`http://`, and for any `<name>::url` whose `git-remote-<name>` helper exists
on PATH — the override closes `ext` by name and cannot close a set that is
whatever the machine happens to carry.

## Root cause (if known)

`is_git` accepts any `word:...` form as a git source with no scheme
allowlist, and `git_identity` normalises on the substring after the first
`://`, so the scheme plays no part in deciding which source an entry names.
That is deliberate for the case it was designed for — one repository written
four ways is one source (ADR-004) — and it is what lets a scheme nobody
vetted inherit the base pack's standing.

## Proposed fix / workaround

- Fix: allowlist rather than denylist, in both places it can be done.
  `-c protocol.allow=never` with explicit `always` for `https`, `ssh` and
  `file` closes every unknown helper at once, where naming `ext` closes one.
  Refusing an unsupported scheme in `PackSource::parse` gives the better
  error and refuses before anything spawns. Either narrows what a manifest
  may say, which [C001](../contracts/C001-content-packs.md) documents as
  `parse`'s rejections, so it is an interface decision — and the reason this
  half stayed unscheduled while the other landed.
- Workaround: read the manifest of a repository before running `sync` in it.

## Regression risk

`pack/source.rs`'s source recognition and `pack/fetch.rs`'s argument vectors;
an allowlist risks refusing a spelling someone legitimately uses, so the
existing `only_the_shorthand_is_expanded_for_git` cases should all still pass.
A test would assert an `ext::` source is refused before any command runs.
