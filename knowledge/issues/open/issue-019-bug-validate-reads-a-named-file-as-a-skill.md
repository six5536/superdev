---
type: BugReport
id: issue-019-bug-validate-reads-a-named-file-as-a-skill
title: validate reads a file named on the command line as a skill, whatever it is
description: superdev validate knowledge/architecture.md reports nine errors about missing skill blocks, because a named path takes the grammar's fallback kind; the document is never checked against the schema its type names, so the check a path argument most obviously invites is the one it cannot run.
lifecycle: open
links:
  - rel: references
    to: contract-002-cli-superdev
    note: The CLI contract governs what validate promises a path argument.
---

# Bug: validate reads a file named on the command line as a skill

## Summary

`superdev validate <path>` checks the named file as a skill whatever it is,
so a knowledge concept is reported as a malformed skill — and the check a
reader naming one document actually wants, against the schema its `type`
names, never runs at all.

## Environment

- Version/commit: superdev 0.2.0, grammar 2.0, after P008
- Platform: any; the behaviour is in path handling and is pure

## Steps to reproduce

1. Run `superdev validate` with no arguments — PASS, no errors.
2. Run `superdev validate knowledge/architecture.md`.
3. Read the report.

## Expected behaviour

The named document is checked as what it is, with full parity to the bare
run. These sentences are the acceptance criteria:

1. WHEN validate is invoked with a path to a file whose frontmatter `type`
   names a schema, THE SYSTEM SHALL report for that file exactly the
   findings a bare run reports for it — schema, filing and link findings
   alike — and no findings about any other file.
2. WHEN validate is invoked with a path to a frontmatter-less file a
   schema names by `target-files` glob (README.md, CHANGELOG.md), THE
   SYSTEM SHALL check it against that schema, never the skill grammar.
3. WHEN validate is invoked with a path to a file whose `type` names no
   schema, THE SYSTEM SHALL report that fault as the bare run does.
4. WHEN validate is invoked with a path to a file with no frontmatter
   that no glob and no grammar kind claims positively, THE SYSTEM SHALL
   check it as the grammar's fallback kind — a skill outside the roots
   stays checkable, as today.
5. IF the named path cannot be read, THEN THE SYSTEM SHALL fail naming
   the path.

## Actual behaviour

```
✗ [error] knowledge/architecture.md: missing <bootstrap_actions> block
✗ [error] knowledge/architecture.md: missing <rules> block
FAIL (9 error(s), 0 warning(s))
```

Nine errors, all of them about a skill's element vocabulary, about a document
that is not a skill and has just passed a whole-repository run.

## Root cause (if known)

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

## Proposed fix / workaround

- A named file whose frontmatter carries a `type` is a document: resolve it
  through the schema set and skip `detect_kind` entirely. The fallback stays
  for files that carry no frontmatter, which is what it was for.
- Build the candidate list for a named path too, so the schema half is
  reachable without a whole-repository run.
- Workaround: run `superdev validate` with no arguments. The whole-repository
  run is correct; it is only the path argument that misreads.

## Regression risk

The fallback is what lets `validate <a-skill-outside-the-roots>` work, and a
test covers that path; the change must leave it. The whole-repository run
does not use the fallback at all, so the blast radius is the path-argument
branch only, which the CLI end-to-end tests exercise directly.

## Comments

2026-08-31, framing: the user chose full parity over a self-contained
schema-only check — a named document run loads the knowledge and the
schema set, so the file gets exactly the findings a bare run gives it,
link resolution included. The path argument's promise is public CLI
behaviour, governed by [contract-002][sokf:contract-002-cli-superdev].

<!-- sokf:links -->
[sokf:contract-002-cli-superdev]: /knowledge/contracts/public/active/contract-002-cli-superdev.md
