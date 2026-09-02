---
type: BugReport
id: issue-007-bug-a-pack-source-reaches-git-with-no-scheme-check
title: A pack source's scheme is unchecked, so the base pack can be fetched over a transport anyone on-path can answer
description: "superdev allowlisted no scheme, so git:// and http:// normalised onto the default identity and a cloned manifest could have the base pack fetched over an unauthenticated transport; fixed in P005 slice 1 and ADR-012, which allowlist https, ssh and file at parse."
lifecycle: done
links:
  - rel: references
    to: contract-007-interface-pack-resolution
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

What remains is below, and it now has the interface decision it was always
going to need.

## Decided

[ADR-012][sokf:adr-012-pack-source-schemes-are-allowlisted]. A pack
source may name only `https`, `ssh` or `file`, allowlisted in both places
this issue said it could be:

- `PackSource::parse` refuses a transport outside that set, naming the source,
  before anything is spawned — and refuses a `<name>::<address>` remote helper
  as one, whatever its address, since a helper names a program rather than a
  protocol. The `github:`/`gitlab:` shorthand is https and the scp form is
  ssh, so no legitimate spelling changes and the existing
  `only_the_shorthand_is_expanded_for_git` cases all still pass.
- The git overrides become `-c protocol.allow=never`, `always` for the same
  three, and `never` naming `git`, `http` and `ext`. Where parse refuses what
  a manifest may *say*, this refuses what git may *do*.

Which half carries the weight came out of measuring rather than reasoning, and
it is the opposite of the obvious answer. Git resolves `protocol.<name>.allow`
ahead of `protocol.allow` whatever their sources, so a user config carrying
`protocol.ext.allow = always` beats `-c protocol.allow=never` on the command
line — `ext::touch pwned` runs — while `-c protocol.ext.allow=never` refuses
it. The same config with `protocol.git.allow = always` gets `git://` past the
blanket. And it runs the other way too: `url.<base>.insteadOf` rewrites a URL
after superdev has handed it over, so a plain `https://` source that `parse`
approves becomes an `ext::` command under a config that asks for it — which
`parse` cannot see and the named refusal still stops.

So the two halves cover different failures and the guarantee is the
conjunction: `parse` bounds what a manifest may name and no config can lift
it; the named refusals bound what git may end up doing with it; the blanket
covers every helper the machine has not spoken about.

`identity` keeps ignoring the scheme. It no longer has a wrong answer to give,
because a source with an unsupported transport never reaches it.

An escape hatch was considered and rejected: the two schemes anyone would
re-admit are precisely the two with no authentication.

## Summary

Against [C007][sokf:contract-007-interface-pack-resolution].

`[[packs]].source` is passed to `git clone` as the URL with no validation of
its scheme and no `--` end-of-options separator. Git's `ext::` transport takes
a **command** as its connection, so a manifest naming one makes `superdev sync`
run it — before any digest is verified, since verification checks the result of
a fetch that has already happened.

The manifest is a committed file, so this is reachable by cloning a branch and
running `sync` in it: a fork, a contributor's PR, any untrusted project. It
narrows the local-by-default guarantee in
[security-requirements][sokf:security-requirements] further than that concept
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

1. `RS_c1` In a scratch repo, `superdev init`, then point the pack entry at the
   default repository over an unauthenticated transport:
   ```toml
   [[packs]]
   source = "git://github.com/six5536/superdev"
   rev = "assets-v0.2.0"
   ```
2. `RS_c2` `superdev sync`

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
  may say, which [C007][sokf:contract-007-interface-pack-resolution] documents as
  `parse`'s rejections, so it is an interface decision — and the reason this
  half stayed unscheduled while the other landed.
- Workaround: read the manifest of a repository before running `sync` in it.

## Regression risk

`pack/source.rs`'s source recognition and `pack/fetch.rs`'s argument vectors;
an allowlist risks refusing a spelling someone legitimately uses, so the
existing `only_the_shorthand_is_expanded_for_git` cases should all still pass.
A test would assert an `ext::` source is refused before any command runs.

<!-- sokf:links -->
[sokf:adr-012-pack-source-schemes-are-allowlisted]: /knowledge/adrs/active/adr-012-pack-source-schemes-are-allowlisted.md
[sokf:contract-007-interface-pack-resolution]: /knowledge/contracts/internal/active/contract-007-interface-pack-resolution.md
[sokf:security-requirements]: /knowledge/security-requirements.md
