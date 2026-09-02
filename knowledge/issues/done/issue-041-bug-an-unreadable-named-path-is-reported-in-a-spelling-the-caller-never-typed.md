---
type: BugReport
id: issue-041-bug-an-unreadable-named-path-is-reported-in-a-spelling-the-caller-never-typed
title: An unreadable named path is reported as an absolute platform-separated path, not the one the caller typed
description: A path named on the command line that cannot be read is reported absolutised and with the platform's separator, so on Windows the caller is handed back a spelling they never typed and one that no finding in the same report uses; fixed by spelling the error the way findings are spelled.
lifecycle: done
---

# Bug: an unreadable named path is reported in a spelling the caller never typed

## Resolved

The read error for a named path now carries the repository-relative,
forward-slashed spelling that `relative` already computed one line above
it, which is what every finding in the same report uses. The test was
strengthened from a substring check to the exact opening, so it fails on
Linux too rather than only on Windows.

## Summary

`superdev validate no/such/file.md` reports the path absolutised and
with the platform's separator. Every finding in the same report spells a
path repository-relative with forward slashes, so one report carries two
spellings, and on Windows neither matches what the caller typed.

## Environment

- Version/commit: 0.2.0 / f8e5e97
- Platform: every platform reports the absolute path; only Windows
  changes the separators with it

## Steps to reproduce

1. `RS_c1` Run `superdev validate no/such/file.md` from a repository root.
2. `RS_c2` Read the error.

## Expected behaviour

The error names the path the caller gave: `no/such/file.md`, the same
spelling a finding about that file would carry.

## Actual behaviour

The path is absolutised, and on Windows separated with backslashes:

```text
Linux:   error: /workspaces/superdev/no/such/file.md: No such file or directory (os error 2)
Windows: error: D:\a\superdev\superdev\no\such\file.md: The system cannot find the path specified. (os error 3)
```

## Root cause (if known)

`validate_repo` normalises every named path against the repository root
before reading it, so the `Error::Io` a failed read carries holds the
absolute spelling. Findings do not go through that path: they are spelt
by `relative`, which strips the root and forward-slashes what is left.
The named-path loop already computes that spelling as `name` on the line
above the read, and the error did not use it.

The test asserted the message *contains* `no/such/file.md`, which the
absolute path does wherever `/` is the separator — so it passed on Linux
and macOS by accident and was only ever going to fail on Windows.

## Proposed fix / workaround

- Fix: report the read error with the spelling `relative` already
  produced, so the error and the findings agree.
- Fix: assert the exact opening of the message rather than a substring,
  so the test fails on every platform when the spelling is wrong.
- Workaround: none needed; the run still fails with the right exit code.

## Regression risk

Only the named-path read reports through this route, and the
strengthened test covers it on every platform. The change touches no
finding's spelling, which the golden snapshots pin.

## Comments

Found by CI on [PR #7](https://github.com/six5536/superdev/pull/7), which
fixed [I039][sokf:issue-039-bug-validate-fix-refuses-to-refile-under-a-symlinked-root]
and [I040][sokf:issue-040-bug-validate-reports-findings-on-a-windows-checkout-that-linux-does-not].
The Windows job had been aborting at test 12 on I040 and never reached
this one, which sits at test 61 — so this defect was masked by that one
rather than introduced by its fix.

<!-- sokf:links -->
[sokf:issue-039-bug-validate-fix-refuses-to-refile-under-a-symlinked-root]: /knowledge/issues/done/issue-039-bug-validate-fix-refuses-to-refile-under-a-symlinked-root.md
[sokf:issue-040-bug-validate-reports-findings-on-a-windows-checkout-that-linux-does-not]: /knowledge/issues/done/issue-040-bug-validate-reports-findings-on-a-windows-checkout-that-linux-does-not.md
