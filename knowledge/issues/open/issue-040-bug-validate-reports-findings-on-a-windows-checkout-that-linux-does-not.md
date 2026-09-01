---
type: BugReport
id: issue-040-bug-validate-reports-findings-on-a-windows-checkout-that-linux-does-not
title: validate reports findings against the live repository on Windows that Linux does not
description: On a Windows checkout `superdev validate` exits 1 against its own knowledge, reporting four types as naming no schema and three links blocks as ungenerated; the four schemas are on file and the same run passes on Linux, and the Windows CI job has been red since before P021.
lifecycle: open
---

# Bug: validate reports findings on a Windows checkout that Linux does not

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

Seven findings appear that Linux does not produce:

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

`knowledge/schemas/` carries `security-requirements.md`,
`software-components.md`, `technology-stack.md` and
`testing-strategy.md`, so all four schemas the run says are missing are
present.

## Root cause (if known)

Not yet known. The leading hypothesis is line endings: `.gitattributes`
forces LF on `.claude/skills/**`, `.agents/**`, `knowledge/templates/**`
and `pack/**`, because those are compared byte-for-byte against content
embedded in the binary, and covers neither `knowledge/schemas/**` nor
the knowledge documents themselves. A Windows checkout therefore gives
those files CRLF, and both symptoms are consistent with that — a links
block compared line-for-line against a regenerated one differs on every
line, and a schema whose parse depends on an exact line shape
registers no type. The four affected concepts sharing a `sources:`
block and a footnote is the part the hypothesis does not yet explain.

Confirm by running `superdev validate --json` on a Windows checkout
made with `core.autocrlf=false` and comparing the finding list to the
default checkout's.

## Proposed fix / workaround

- Fix: establish whether line endings are the mechanism, by the
  comparison above, before changing anything.
- Fix: if they are, either normalise line endings on read in the
  validator, or extend `.gitattributes` to the knowledge tree — the
  first travels to a managed repository and the second does not.
- Workaround: check the repository out with `core.autocrlf=false`.

## Regression risk

The reading path is shared by every check the validator runs, so a
change to it touches all of them; the three Windows CLI tests catch a
recurrence, and they only catch it because CI runs Windows — which is
the reason the job must be returned to green rather than muted.
