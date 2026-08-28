---
type: FeatureRequest
id: issue-006-feature-request-content-packs-are-undocumented-for-users
title: Content packs are absent from the user documentation, and the update command now describes itself wrongly
description: Neither the README nor the CLI help nor the man page mentioned packs, so a user could not discover the feature, and update's description still claimed it moved pins to this binary's defaults; fixed in slice 18, which added a packs section and corrected update on every surface.
status: stable
tags: [done]
links:
  - rel: references
    to: spec-014-content-packs
---

# Feature: content packs are undocumented, and `update` describes itself wrongly

## Resolved

P003 slice 18. `update`'s description says what it does on every surface it
reaches — `--help`, the man page, the zsh completions and the README all come
from two lines, and the same sentence had also survived in `api-contracts.md`,
which agents read in preference to the README. The README gained a section on
where content comes from, and the man page's top-level description says packs
exist, since it carries no long description for any subcommand.

Both are held by tests: prose goes stale in silence, and this one had gone
stale in five places from one edit nobody made.

Two things learned in the fixing, recorded because the next person will meet
them. clap takes a doc-comment paragraph whole and `wrap_help` is not enabled,
so a description long enough to be useful runs off the terminal and breaks the
help table's alignment — the long ones are hand-wrapped. And the man page is a
one-line index per subcommand, so detail belongs in the top-level description
rather than in a subcommand's summary.

## Summary

Against [S014](../specs/spec-014-content-packs.md).

Two gaps found at acceptance, one an omission and one an inaccuracy.

The feature is **undocumented for users**. `README.md` does not mention packs,
`superdev --help` does not, and neither does the man page — zero occurrences in
any of them. Everything written about packs during delivery went to the
knowledge (internal) or CONTRIBUTING (contributors). Someone who installs
superdev has no way to learn that `[[packs]]` exists, that content now releases
separately from the binary, or that `update` will move their pin.

Worse, `update` now **describes itself wrongly**, in the shipped binary:

```
update    Move version pins to this binary's defaults, then sync
```

That was true before this feature. It is not true now: `update` asks the
default pack source for its newest release and moves the pin there, *ahead* of
this binary's default, which is the whole point of
[ADR-009](../decisions/adr-009-update-queries-default-source.md). The same
sentence is in `README.md` and in the man page, and it tells a user the one
verb that reaches the network does not.

## Motivation

No user-facing mention of the feature exists, and three places state the
opposite of what `update` does.

The inaccurate sentence is the doc comment at
`crates/app/superdev/src/main.rs:48`, which clap renders into `--help` and the
man page; `crates/app/superdev/src/manage.rs:299` carries the same wording, and
`README.md` repeats it. No slice in P003 owned the README, and the help text
was not revisited when slice 11 changed what `update` does.

## Proposed behaviour

A user can discover packs from the product's own documentation, and every
description of `update` says that it may reach the network and move the pack
pin past what the binary carries.

## Alternatives considered

- Document packs in CONTRIBUTING alone — where everything written during
  delivery had already gone, and where a user who installed the binary
  never looks.
- Leave `update`'s description and correct it at the next release — the
  sentence tells a user that the one verb reaching the network does not,
  which is the kind of wrong that costs trust rather than time.

## Scope

- Fix: correct the `update` description in `main.rs` (and the rustdoc in
  `manage.rs`) so it says the pack pin may move to the source's newest release;
  add a packs section to `README.md` covering the `[[packs]]` entry, layering
  and base replacement, the two release series, and that `update` is the verb
  that reaches out. Keep it to the README's register — short, and pointing at
  the canonical knowledge for the rest.
- Workaround: none; the information exists only in `knowledge/` and
  CONTRIBUTING.
