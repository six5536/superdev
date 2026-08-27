---
type: Issue
id: issue-012-five-decidable-findings-only-warn
title: Five findings the repository alone can decide are only warnings, and go unread
description: Broken links, missing resources, missing sources, missing index targets and unjoined footnotes are all decidable from the tree, but SPEC §11 makes them warnings; the canonical knowledge carried 39 of them unactioned until someone happened to look.
status: draft
tags: [needs-triage]
links:
  - rel: relates-to
    to: adr-017-aokf-conformance-is-pass-or-fail
---

# Bug: five findings the repository alone can decide are only warnings, and go unread

## Summary

The validator has eight warning classes. Three of them report something the
tool can see but cannot judge — a frontmatter key outside the portable spec, a
custom `rel`, a description over the host's truncation limit — and those are
warnings for a good reason: the answer depends on where a skill is published,
on what a consumer does with an unknown relationship, or on a limit that only
degrades. The other five are decidable from the repository alone. A file
exists or it does not; a footnote label joins a source or it does not. They
warn anyway, so nothing fails, so nobody reads them.

## Environment

- Version/commit: superdev 0.2.0, AOKF 0.3
- Platform: any; every one of these checks is pure and reads only the tree

## Steps to reproduce

1. Add `[nowhere](does-not-exist.md)` to any concept under `knowledge/`.
2. Run `cargo run --quiet -- aokf validate knowledge`.
3. Note the exit code.

## Expected behaviour

A link to a file that is not there fails the run, the way every other
repository-decidable rule does. The tree is the whole input; there is nothing
else the answer could depend on.

## Actual behaviour

`PASS`, exit 0, with the finding printed as a warning among any others. The
edit-time hook exits 0 too, so the agent that wrote the link is never told.

This is not hypothetical. Until it was noticed during the schema migration the
knowledge carried **39** warnings, every one of them
`sources[0].resource does not exist: /knowledge/templates/<name>.md`. They had
accumulated because the schemas cite the templates they replaced. They were
real, they were trivial to fix once seen, and they were invisible for as long
as they existed because nothing ever failed on them.

## Root cause (if known)

SPEC §11 closes with:

> Consumers must be permissive. In particular, never reject knowledge for
> missing optional fields, unknown `type` values, unknown frontmatter keys,
> unknown `rel` values, broken links, or a missing `index.md` or manifest.

`broken links` is what puts four of the five in the warning tier, and §10 item
5 lists them as warn-only accordingly. The fifth — a footnote label with no
matching `sources[].id` — follows the same list.

The permissiveness line is aimed at *consumers*: a reader of knowledge should
not refuse to display it over a dangling link. It has been read as a rule for
*validators* too, which is a different job. A validator that never fails is
not being permissive, it is being ignored.

The five, all at `crates/lib/superdev-core/src/aokf/validate.rs`:

- `broken body link: {target}`
- `` `resource` path does not exist: {resource} ``
- `sources[N].resource does not exist: {resource}`
- `index entry points at missing file: {target}`
- `footnote [^label] has no matching sources[].id`

## Proposed fix / workaround

Split the tier by decidability rather than by tradition. A finding the
repository alone can settle is an error; a finding whose answer lies outside
the repository stays a warning. That keeps the three that earn the tier — the
portability warning, the non-core `rel`, the soft length limit — and moves the
five.

Two of the five want thought before they move, and they are the reason this is
an issue rather than a patch. A broken body link and a missing index target
are exactly what ordinary work-in-progress produces: the link is written
before the file it points at lands. Making them fatal means the PostToolUse
hook blocks the agent mid-thought, several edits before the state it is
working towards. That is an argument about those two specifically, not about
the class, and it may be answered by when the check runs rather than by how
loudly it complains.

This changes SPEC §11, so it wants the treatment ADR-017 had rather than a
quiet edit — see
[the conformance decision](../decisions/D017-aokf-conformance-is-pass-or-fail.md),
which took the same kind of question about the same section.

Workaround until then: read the warnings. The canonical knowledge currently sits at zero,
so a new one is visible; that is a property of having just cleared them, not
of the design.

## Regression risk

Every repository superdev manages gets stricter at once, and any that has been
carrying dangling links starts failing its pre-PR check. That is the point,
but it is a breaking change to what `validate` accepts and belongs in the same
release note as any other.

The five checks have no covering tests of their own beyond the parity goldens,
which pin them as warnings — `broken-links.golden.json` records
`"severity": "warning"` for exactly this case. Moving them changes those
goldens, and the goldens cannot be regenerated, so the change has to be made
as a recorded projection the way ADR-017's was.
