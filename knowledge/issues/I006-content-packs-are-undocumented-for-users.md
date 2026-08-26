---
type: Issue
id: issue-006-content-packs-are-undocumented-for-users
title: Content packs are absent from the user documentation, and the update command now describes itself wrongly
description: Neither the README nor the CLI help nor the man page mentions packs, so a user cannot discover the feature; and update's one-line description still says it moves pins to this binary's defaults, which stopped being true when it began querying the default source.
status: stable
tags: [done]
links:
  - rel: references
    to: spec-content-packs
---

# Bug: content packs are undocumented, and `update` describes itself wrongly

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

Against [S014](../specs/S014-content-packs-design.md).

Two gaps found at acceptance, one an omission and one an inaccuracy.

The feature is **undocumented for users**. `README.md` does not mention packs,
`superdev --help` does not, and neither does the man page — zero occurrences in
any of them. Everything written about packs during delivery went to the
knowledgebase (internal) or CONTRIBUTING (contributors). Someone who installs
superdev has no way to learn that `[[packs]]` exists, that content now releases
separately from the binary, or that `update` will move their pin.

Worse, `update` now **describes itself wrongly**, in the shipped binary:

```
update    Move version pins to this binary's defaults, then sync
```

That was true before this feature. It is not true now: `update` asks the
default pack source for its newest release and moves the pin there, *ahead* of
this binary's default, which is the whole point of
[ADR-009](../decisions/D009-update-queries-default-source.md). The same
sentence is in `README.md` and in the man page, and it tells a user the one
verb that reaches the network does not.

## Environment

- Version/commit: 0.2.0 / P003 complete (`e1ac431`)
- Platform: all

## Steps to reproduce

1. `superdev --help` — grep for `pack`: no matches.
2. `superdev man` — grep for `pack`: no matches.
3. `grep -i pack README.md` — no matches.
4. Read `update`'s description in any of the three.

## Expected behaviour

A user can discover packs from the product's own documentation, and every
description of `update` says that it may reach the network and move the pack
pin past what the binary carries.

## Actual behaviour

No user-facing mention of the feature exists, and three places state the
opposite of what `update` does.

## Root cause (if known)

The inaccurate sentence is the doc comment at
`crates/app/superdev/src/main.rs:48`, which clap renders into `--help` and the
man page; `crates/app/superdev/src/manage.rs:299` carries the same wording, and
`README.md` repeats it. No slice in P003 owned the README, and the help text
was not revisited when slice 11 changed what `update` does.

## Proposed fix / workaround

- Fix: correct the `update` description in `main.rs` (and the rustdoc in
  `manage.rs`) so it says the pack pin may move to the source's newest release;
  add a packs section to `README.md` covering the `[[packs]]` entry, layering
  and base replacement, the two release series, and that `update` is the verb
  that reaches out. Keep it to the README's register — short, and pointing at
  the knowledgebase for the rest.
- Workaround: none; the information exists only in `knowledge/` and
  CONTRIBUTING.

## Regression risk

`main.rs`'s clap derive feeds `--help`, the man page and the completions, so
one edit moves all three. A test asserting the help text names the network
behaviour would keep it honest.
