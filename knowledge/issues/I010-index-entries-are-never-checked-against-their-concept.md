---
type: Issue
id: issue-010-index-entries-are-never-checked-against-their-concept
title: An index entry may say anything about a concept, and nothing notices
description: SPEC §9 says an index entry should carry the linked concept's description, but check_indexes only tests that the target exists, so an index can drift from every concept it lists — or hold the only copy of something — and validate still passes.
status: draft
tags: [needs-triage]
---

# Bug: an index entry may say anything about a concept, and nothing notices

## Summary

`superdev aokf validate` checks that an `index.md` entry points at a file that
exists, and nothing else about what the entry says. SPEC §9 requires that
"entries should carry the linked concept's `description`", so an index may
describe a concept in terms the concept itself does not use, and the bundle
still passes. It bites in two directions: an entry silently goes
stale when the concept is reworded, and an entry can become the only home of a
fact — which matters because §9 also says indexes may be generated, so a
regeneration would destroy it.

## Environment

- Version/commit: superdev 0.2.0 / dff3ac2
- Platform: any; the check is pure and does no IO beyond resolving targets

## Steps to reproduce

1. Edit any entry in `knowledge/decisions/index.md` so its text after the
   `-` no longer matches that decision's `description` — swap in a sentence
   of your own.
2. Run `cargo run --quiet -- aokf validate knowledge`.
3. Read the report.

## Expected behaviour

A warning naming the index, the entry, and the fact that its text does not
match the linked concept's `description`, in the manner of the existing
`index entry points at missing file:` warning.

## Actual behaviour

`PASS (0 error(s), 0 warning(s))`. The edited entry is reported
nowhere.

The same run before this was noticed carried fourteen such divergences across
five indexes, none of them visible: five were stale wording, and nine were the
`knowledge/issues/index.md` entries, which held each issue's resolution while
the concept described only the symptom.

## Root cause (if known)

`crates/lib/superdev-core/src/aokf/validate.rs:525`. `check_indexes` walks the
links in each index and resolves them, warning only when a target does not
exist:

```rust
/// `index.md` entries point at files that exist.
fn check_indexes(bundle: &Bundle, repo_root: &Path, findings: &mut Vec<Finding>) {
```

That is faithful to SPEC §10, whose warn list names exactly one index rule —
missing targets. The §9 requirement about descriptions is stated where nothing
implements it.

## Proposed fix / workaround

Extend `check_indexes` to compare each entry's text against the linked
concept's `description` and warn on a mismatch. The function already resolves
the target path, and `bundle.concepts` is already loaded, so the lookup adds
no IO. A warning rather than an error: §11 requires consumers to be permissive,
`index.md` is a reserved file and not a concept, and §9 says "should".

Two details the implementation has to settle, both visible in the current
indexes: entries de-capitalise the concept's first letter as a house style, so
the comparison is case-insensitive on the first character; and the comparison
should ignore a trailing full stop.

Workaround until then: none in the tool. The divergences in this repo were
found by a throwaway script and fixed by hand.

## Regression risk

`check_indexes` has no covering tests today, so the change lands on untested
ground and should bring its own. The blast radius is one function reached only
from `validate`, and a warning never fails a bundle — only a fatal finding
does.
