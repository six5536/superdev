---
type: BugReport
id: issue-039-bug-validate-fix-refuses-to-refile-under-a-symlinked-root
title: validate --fix refuses to refile a document when the knowledge root resolves through a symlink
description: The containment guard canonicalises the destination path, which does not exist yet, so the fallback keeps the uncanonicalised form and fails the prefix check against a canonical root — refiling is refused on macOS, where /var and /tmp are symlinks; fixed by resolving the nearest existing ancestor, which returned the macOS CI job to green.
lifecycle: done
---

# Bug: validate --fix refuses to refile under a symlinked root

## Resolved

Fixed at the root: a `resolved` helper canonicalises the nearest existing
ancestor and re-appends the rest, so a path that does not exist yet is
still compared in canonical form. `move_within` and `write_within` both
use it, because a new file has the same defect a moved one does.

A `..` in the part that does not exist is refused rather than resolved,
and `resolved` returns `Option` so both callers treat "cannot resolve" as
"refuse". The first fix ended the walk on the raw spelling instead and
its comment claimed that was safe; it is not. `canonicalize` resolves
none of `root/gone/../../elsewhere`, the raw spelling literally begins
with `root`, so a lexical `starts_with` passes it and the filesystem
lands the write outside the knowledge. The same hole was in the guard
this issue replaced, so it is as old as the guard. A review of the pull
request found it, and a test now drives four escape spellings through
both guards.

## Summary

`superdev validate --fix` refuses to move a misfiled document when the
knowledge root reaches it through a symlink, reporting the move as an
escape from the knowledge. Every macOS user working under `/tmp` or
`/var` hits it, and it has held the macOS CI job red for at least a
day before this acceptance ran.

## Environment

- Version/commit: 0.2.0 / 19ac275 (`main`), and every commit back to
  `d07365a`, which introduced the guard
- Platform: macOS (`/var` and `/tmp` are symlinks to `/private/...`);
  any platform where the knowledge root is reached through a symlink

## Steps to reproduce

1. `RS_c1` On macOS, create a knowledge tree under a temporary directory, so
   its path runs through `/var/folders/...`.
2. `RS_c2` Write a document whose `lifecycle` disagrees with its folder — an
   issue reading `lifecycle: open` while it sits in `issues/done/`.
3. `RS_c3` Run `superdev validate --fix` against that root.

In CI the same path is driven by the test
`validate::fix::tests::the_pass_files_by_lifecycle_before_repairing_links`,
which fails on every macOS run.

## Expected behaviour

1. `EX_c1` [ubiquitous] The document moves to the folder its `lifecycle` names, as it does on
   Linux, and the guard refuses only a genuine escape from the knowledge
   root.

## Actual behaviour

The move fails and the error names the destination as outside the
knowledge:

```text
thread 'validate::fix::tests::the_pass_files_by_lifecycle_before_repairing_links'
panicked at crates/lib/superdev-core/src/validate/fix.rs:475:47:
called `Result::unwrap()` on an `Err` value: Io {
  path: "/var/folders/df/.../T/.tmpUQULvL/knowledge/issues/open/issue-001-bug-a.md",
  source: Custom { kind: Other, error: "refusing to move outside the SOKF
  knowledge at /private/var/folders/df/.../T/.tmpUQULvL/knowledge" } }
```

The two paths in that message are the same directory: the root is
canonical, the refused path is not.

## Root cause (if known)

`move_within` at `crates/lib/superdev-core/src/validate/fix.rs:292`
checks both ends of the rename:

```rust
let resolved = canonical(path).unwrap_or_else(|| path.to_path_buf());
if !resolved.starts_with(root) {
```

The destination does not exist yet — refiling is what creates it — so
`canonical` fails on it and the fallback keeps the uncanonicalised
path. On macOS the root is canonical (`/private/var/...`) and the
fallback is not (`/var/...`), so the prefix check fails on a
destination inside the knowledge. The source end passes because it
exists and canonicalises. Linux does not expose the defect because
neither form differs there.

## Proposed fix / workaround

- Fix: resolve the destination against its nearest existing ancestor
  rather than falling back to the raw path, so a path that does not
  exist yet is still compared in canonical form.
- Fix: cover it with a test that runs under a symlinked root on every
  platform, rather than relying on macOS to expose it.
- Workaround: run superdev under a path with no symlinked component.

## Regression risk

`move_within` is the only mover in the fix pass, so every lifecycle
refile goes through it; the existing lifecycle test catches a
recurrence on macOS, and a symlinked-root test would catch one
everywhere.
