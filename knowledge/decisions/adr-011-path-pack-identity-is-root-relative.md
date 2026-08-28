---
type: Decision
id: adr-011-path-pack-identity-is-root-relative
title: A Path Pack's Identity Is Relative to the Repo Root
description: A path source's identity is its canonicalised path expressed relative to the repository root with forward slashes, so the committed lock says the same thing on every checkout and every platform.
status: stable
links:
  - rel: implements
    to: spec-014-content-packs
  - rel: relates-to
    to: adr-004-base-pack-identity
---

# ADR-011: A path pack's identity is relative to the repo root

- Status: accepted
- Date: 2026-08-26
- Deciders: project owner

## Context

[The spec](../specs/spec-014-content-packs.md) lets a pack come from a
directory on this machine.
[ADR-004](adr-004-base-pack-identity.md) settled that a source is compared on a
normalised identity, and made a path source's identity its canonicalised
absolute path — the spelling that makes two ways of writing one directory into
one pack.

`.superdev/lock.toml` is committed, and it records that identity. So long as
no repository pinned a path pack, the absolute path never left the machine
that wrote it. Dogfooding superdev onto its own `/pack/` is the first entry
that would be committed, and it writes the author's directory layout into a
tracked file:

```toml
[[packs]]
source   = "./pack"
identity = "/workspaces/superdev/pack"
```

Checked out anywhere else, `sync` rewrites that line to the new location.
`status --drift` still passes, so CI would not catch it; the cost is a tracked
file that churns per checkout and a public repository carrying one
contributor's paths.

The identity is read back for exactly one purpose — matching a fetched git
pack to its lock record — and that path never runs for a directory source,
which is re-read from disk every run. For a path pack the field is written and
never consulted.

## Decision

We will make a path source's identity its canonicalised path expressed
**relative to the repository root**, written with forward slashes. `./pack`
and `pack/` both canonicalise and then relativise to `pack`, so two spellings
of one directory remain one pack, and the committed lock says the same thing
on every checkout and every platform.

A pack outside the root keeps its `..` prefix — `../shared/packs` — rather
than being refused. Where no relative form exists at all, which on Windows
means a different drive, the canonical absolute path stands.

Identity keeps one meaning and one representation: the same value compares
sources within a run, guards against an entry named twice, and is recorded in
the lock. It therefore takes the root as a parameter —
`identity(&self, root: &Path)` — because a relative key cannot be computed
without one, and because a path identity genuinely means nothing apart from
the repository it was taken from. A git source ignores the argument.

Two keys are compared only within a source kind. An absolute path could never
be mistaken for a repository key; a relative one could, and a directory named
`github.com/six5536/superdev` keying as the base pack would silently replace
the embedded content — the failure [ADR-004](adr-004-base-pack-identity.md) calls
the worst available here. A directory and a repository are different sources
however alike their keys read.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Root-relative identity | One identity with one meaning, used by every call site; the committed lock is identical on every checkout and OS; two spellings still collapse to one key | Needs a relativisation step, and a fallback for the one case that has no relative form |
| Omit `identity` for path packs | Smallest change to what is stored; nothing reads it for a path source today | The lock stops recording what a path entry resolved to, and the field becomes optional for one source kind — a schema that has to explain itself |
| Split the two meanings — absolute in memory, relative in the lock | Comparison semantics untouched | One concept with two representations, and a reader has to know which is which at every site |
| Leave it absolute | No change | Commits a contributor's directory layout to a public repository, and every checkout's first `sync` produces a diff nobody intended |
| Refuse a pack outside the root | Every identity is a plain relative path | Forbids a shared pack checked out beside the repo, a use the spec never ruled out, decided here rather than in the spec |

## Consequences

- Positive: a repository may commit a path pack, which is what lets superdev
  resolve its own content from the tree and ship a skill fix without a
  rebuild.
- Positive: forward slashes mean a Windows contributor and a Linux one write
  the same lock line.
- Negative: an identity is no longer meaningful without knowing the root it
  was taken from. That is what the lock already is — a record about one
  repository — but it does mean the value cannot be compared across
  repositories.
- Negative: the Windows different-drive fallback leaves one case where an
  absolute path is still recorded, so the guarantee is very nearly, but not
  quite, unconditional.
- Negative: `identity` grows a parameter that a git source does not use, and
  every call site must have the root to hand. All of them are in the resolver,
  which does.
- Follow-ups: [C001](../contracts/private/contract-001-interface-content-packs.md) states a path
  source's key as its canonicalised absolute path and must be corrected;
  [configuration](../configuration.md) describes the lock's identity field
  and gains the relative form at integrate.
