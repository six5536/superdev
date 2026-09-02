---
type: BugReport
id: issue-031-bug-validate-follows-symlinks-out-of-the-repo
title: validate follows symlinks out of the repo and can echo what it reads
description: The walk and the file reads follow symlinks, so a hostile checkout can point a governed name at any readable file — the PostToolUse hook then reads it automatically, and finding messages that quote source lines can carry excerpts into the transcript.
lifecycle: open
links:
  - rel: references
    to: issue-019-bug-validate-reads-a-named-file-as-a-skill
    note: Found by the security review in I019's acceptance; the walk predates that feature.
---

# Bug: validate follows symlinks out of the repo and can echo what it reads

## Summary

`walk`, `collect` and `read` follow symlinks, so a cloned repository can
plant a link from a governed name to any file the user can read; the
PostToolUse hook validates automatically after edits, and finding
messages that quote source lines can carry excerpts of the target into
an agent transcript. Low severity: it needs a hostile checkout, and the
tool never promises repo-root confinement — the pack side refuses
symlinks (ADR-014), the validator does not. Found by the security
review in the acceptance of
[I019][sokf:issue-019-bug-validate-reads-a-named-file-as-a-skill]; the
walk predates that feature.

## Environment

- Version/commit: feature/validate-path-dispatch head, 2026-09-01
- Platform: any; confirmed live on Linux with a directory symlink to
  /etc

## Steps to reproduce

1. `RS_c1` In a repository, run `ln -s /etc docs-link` where a grammar root or
   a named directory will reach it.
2. `RS_c2` Run `superdev validate docs-link`.
3. `RS_c3` Observe the walk entering the target; some finding messages quote
   the lines they object to.

## Expected behaviour

The validator refuses or skips a symlink that resolves outside the
repository root, naming the path, as the pack walk already does
(ADR-014).

## Actual behaviour

The walk enters the symlink target and reads what it finds; an
unreadable subdirectory fails the run loudly, a readable one is checked
and quoted.

## Root cause (if known)

`walk` in `crates/lib/superdev-core/src/validate/mod.rs` uses
`read_dir` with `path.is_dir()`, and `read` uses `read_to_string`; all
three follow links. The behaviour predates P018 for bare runs; P018
extended the same walk to named directories.

## Proposed fix / workaround

- Fix: check `symlink_metadata` in the walk and the named-file reads;
  skip or refuse a link whose target resolves outside the repository
  root, naming the path.
- Workaround: none needed for trusted checkouts; do not run validate —
  or the hook — in a checkout from an untrusted source.

## Regression risk

The pack walk's symlink refusal (ADR-014) is separate code and stays.
A confinement check must not break the legitimate case the repository
itself uses: `/pack` reaching `superdev-core` by symlink inside the
repo. The validator snapshot suites would catch a skipped file that
should have been read.

<!-- sokf:links -->
[sokf:issue-019-bug-validate-reads-a-named-file-as-a-skill]: /knowledge/issues/done/issue-019-bug-validate-reads-a-named-file-as-a-skill.md
