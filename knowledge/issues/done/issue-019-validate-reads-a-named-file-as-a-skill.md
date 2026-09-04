---
type: Issue
id: issue-019-validate-reads-a-named-file-as-a-skill
title: validate reads a file named on the command line as a skill, whatever it is
description: superdev validate knowledge/architecture.md reports nine errors about missing skill blocks, because a named path takes the grammar's fallback kind; the document is never checked against the schema its type names, so the check a path argument most obviously invites is the one it cannot run.
kind: bug
lifecycle: done
links:
  - rel: references
    to: contract-002-cli-superdev
    note: >
      The validate bullet's PATH sentence now promises bare-run parity
      for a named document (ADR-026), replacing "only what it names is
      read".
---

# Bug: validate reads a file named on the command line as a skill

## Summary

`superdev validate <path>` checks the named file as a skill whatever it is,
so a knowledge concept is reported as a malformed skill — and the check a
reader naming one document actually wants, against the schema its `type`
names, never runs at all.

## Context

Found on superdev 0.2.0, grammar 2.0, after P008, on any platform; the
behaviour is in path handling and is pure.

1. Run `superdev validate` with no arguments — PASS, no errors.
2. Run `superdev validate knowledge/architecture.md`.
3. Read the report.

## Behaviour

The named document is checked as what it is, with full parity to the bare
run. These sentences are the acceptance criteria:

- When validate is invoked with a path to a file whose frontmatter `type`
  names a schema, validate reports for that file exactly the findings a
  bare run reports for it — schema, filing and link findings alike — and
  no findings about any other file.
- When validate is invoked with a path to a frontmatter-less file a schema
  names by `target-files` glob (README.md, CHANGELOG.md), validate checks
  it against that schema, never the skill grammar.
- When validate is invoked with a path to a file whose `type` names no
  schema, validate reports that fault as the bare run does.
- When validate is invoked with a path to a file with no frontmatter that
  no glob and no grammar kind claims positively, validate checks it as the
  grammar's fallback kind — a skill outside the roots stays checkable, as
  today.
- If the named path cannot be read, validate fails naming the path.

Instead, the run reports:

```
✗ [error] knowledge/architecture.md: missing <bootstrap_actions> block
✗ [error] knowledge/architecture.md: missing <rules> block
FAIL (9 error(s), 0 warning(s))
```

Nine errors, all of them about a skill's element vocabulary, about a document
that is not a skill and has just passed a whole-repository run.

Two causes, and the second arrived with P008.

`crates/lib/superdev-core/src/validate/mod.rs` passes `by_fallback: true` for
a file named on the command line, where the directory walk passes `false`.
The fallback kind is `unit`, so any named markdown file is read as a skill.
That was deliberate once — it let a caller check a file the roots did not
reach — and it predates documents having schemas.

The second is the gap: `validate_repo` builds its document candidates only
inside the branch that loads the SOKF knowledge, which a run naming one file
does not enter. So the schema half has nothing to check even if the kind were
right.

## Scope

The fix proposed at filing, and the way round it meanwhile:

- A named file whose frontmatter carries a `type` is a document: resolve it
  through the schema set and skip `detect_kind` entirely. The fallback stays
  for files that carry no frontmatter, which is what it was for.
- Build the candidate list for a named path too, so the schema half is
  reachable without a whole-repository run.
- Workaround: run `superdev validate` with no arguments. The whole-repository
  run is correct; it is only the path argument that misreads.
- Regression risk: the fallback is what lets `validate <a-skill-outside-the-roots>`
  work, and a test covers that path; the change must leave it. The
  whole-repository run does not use the fallback at all, so the blast radius
  is the path-argument branch only, which the CLI end-to-end tests exercise
  directly.

## Resolution

Fixed by P018 on `feature/validate-path-dispatch` (c97b6da..38f9b83):
the named run is the bare pipeline with its report scoped to what the
paths cover, per ADR-026. Acceptance on 2026-09-01 walked all five
criteria on the feature head — parity, glob dispatch, the unknown-type
fault, the preserved fallback and the unreadable-path failure — against
a probe tree and the live repository, with the original repro now
printing `no findings — PASS`. The feature-wide review's ten findings
were resolved before the merge (code-review-003). The security review
judged the diff sound and surfaced two pre-existing low-severity faults,
filed as I031 and I032.

## Comments

2026-08-31, framing: the user chose full parity over a self-contained
schema-only check — a named document run loads the knowledge and the
schema set, so the file gets exactly the findings a bare run gives it,
link resolution included. The path argument's promise is public CLI
behaviour, governed by [contract-002][sokf:contract-002-cli-superdev].

<!-- sokf:links -->
[sokf:contract-002-cli-superdev]: /knowledge/contracts/public/active/contract-002-cli-superdev.md
