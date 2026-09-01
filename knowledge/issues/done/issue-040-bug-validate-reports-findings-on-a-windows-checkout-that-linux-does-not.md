---
type: BugReport
id: issue-040-bug-validate-reports-findings-on-a-windows-checkout-that-linux-does-not
title: validate reports findings against the live repository on Windows that Linux does not
description: On a Windows checkout `superdev validate` exits 1 against its own knowledge, reporting four types as naming no schema and three links blocks as ungenerated; the four schemas are on file and the same run passes on Linux; the cause was CRLF line endings, fixed by normalising them where the validator reads, which returned the Windows CI job to green.
lifecycle: done
---

# Bug: validate reports findings on a Windows checkout that Linux does not

## Resolved

The hypothesis held: line endings, confirmed by converting the knowledge
tree to CRLF on Linux, which reproduces the failure exactly —
`documents: 0 checked against 56 schemas` and 318 errors.

Fixed by making CRLF and LF the same to every comparison, rather than by
normalising anywhere. Nothing rewrites what it reads, and no reader has
to remember a normalising step: a line is the same line whatever ends it.

- `validate::lines` splits text into lines with the terminator already
  gone, and every line-based check in both halves of the validator goes
  through it. It keeps the empty final element `split('\n')` produces,
  which `str::lines` drops, because the checks index by line number.
- The generated-block check and the repair pass compare line by line
  rather than byte by byte, so a document whose only difference from the
  repair is what ends its lines needs no repair — `validate --fix` leaves
  a CRLF file untouched instead of rewriting it to LF.
- The drift tests scan for a fence a line at a time. Four of them split
  on the literal ```` ```tag\n ````, which finds nothing in a CRLF file;
  the CLI drift test already scanned line-wise and is the shape the other
  four now take.
- Where a test compares a file against a literal written with LF, the
  comparison makes the two the same. That is the only normalising left,
  and it is at the comparison, not on the way in.

`.gitattributes` was rejected as the fix: it would settle superdev's own
checkout and travels to none of the repositories superdev governs.

An earlier attempt normalised at the read instead. It was replaced: it
put the obligation on every reader, and the drift tests — which read the
same files outside the validator — did not have it, so a Windows
checkout still failed on the contract they parse.

## Summary

`superdev validate` exits 1 against superdev's own knowledge on
Windows and 0 on Linux, from the same commit. It reports four document
types as naming no schema when all four schemas are on file, and three
generated links blocks as ungenerated. Every Windows user validating a
knowledge tree is affected, and it has held the Windows CI job red for
at least a day before this acceptance ran.

## Environment

- Version/commit: 0.2.0 / 19ac275 (`main`), and earlier — the same
  failure class appears on the run of 2026-08-31
- Platform: `windows-latest` in CI; Linux and macOS do not report these
  findings

## Steps to reproduce

1. Check the repository out on Windows with the default
   `core.autocrlf`.
2. Run `superdev validate --json`.
3. Observe exit code 1 and the findings below.

In CI the same path is driven by `validate_passes_the_live_repository`,
`validate_json_is_machine_readable` and
`a_named_runs_findings_equal_the_bare_runs_for_that_file` in
`crates/app/superdev/tests/cli.rs`.

## Expected behaviour

`validate` reports the same findings on every platform for the same
tree, and passes clean against superdev's own knowledge as it does on
Linux.

## Actual behaviour

The run header reports `documents: 0`, and every typed document is
reported as naming no schema — no schema registers at all. The CI log
truncates the finding list, so only its tail is visible:

```text
knowledge/security-requirements.md: type `SecurityRequirements` names no schema
knowledge/software-components.md:   type `SoftwareComponents` names no schema
knowledge/technology-stack.md:      type `TechnologyStack` names no schema
knowledge/testing-strategy.md:      type `TestingStrategy` names no schema
knowledge/software-components.md: the <!-- sokf:links --> block is not in
  generated form (run `superdev validate --fix`)
knowledge/technology-stack.md:    (the same)
knowledge/testing-strategy.md:    (the same)
```

Every one of those schemas is on file. Reading the tail as the whole
list is what first suggested four particular documents were special;
they are not, and nothing about them is.

## Root cause (if known)

Line endings. `.gitattributes` forces LF on `.claude/skills/**`,
`.agents/**`, `knowledge/templates/**` and `pack/**`, because those are
compared byte-for-byte against content embedded in the binary, and
covers neither `knowledge/schemas/**` nor the knowledge documents. A
Windows checkout therefore hands both CRLF, and every parser behind the
validator is written against LF: a schema whose contract block is
scanned line by line registers no type, and a generated block compared
against a regenerated one differs on every line.

Confirmed by converting this repository's knowledge tree to CRLF on
Linux and running `validate` against it, which reproduces the failure
exactly — `documents: 0 checked against 56 schemas`, every type
ungoverned, every generated block reported ungenerated.

## Proposed fix / workaround

- Fix: make CRLF and LF the same to every comparison — read a line at a
  time and compare lines, rather than normalising on the way in. Nothing
  rewrites what it reads, and no reader can forget a step it does not
  have to take.
- Rejected: extending `.gitattributes`, which would settle superdev's own
  checkout and travels to none of the repositories superdev governs.
- Rejected: normalising at the read, which puts the obligation on every
  reader — the drift tests read the same files outside the validator and
  would still fail.
- Workaround: check the repository out with `core.autocrlf=false`.

## Regression risk

The reading path is shared by every check the validator runs, so a
change to it touches all of them; the three Windows CLI tests catch a
recurrence, and they only catch it because CI runs Windows — which is
the reason the job must be returned to green rather than muted.
