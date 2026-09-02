---
type: BugReport
id: issue-043-bug-the-cli-contracts-json-keys-are-bound-by-no-test
title: The CLI contract's declared JSON keys are compared to nothing, and two keys the binary emits went undeclared
description: The CLI drift test binds commands, flags, arguments and exit codes, and not the `json` block; `documents` and `schemas` have been emitted without being declared, which nothing noticed.
lifecycle: wontfix
links:
  - rel: references
    to: issue-049-feature-request-a-contract-cannot-point-at-its-definition
    note: Dissolves this — the copy the test compared no longer exists.
  - rel: references
    to: issue-035-feature-request-a-contract-does-not-define-its-interface
    note: Criterion 4's drift test covers the command tree and not the JSON the same contract declares.
---

# Bug: the CLI contract's JSON keys are bound by no test

## Won't fix

Dissolved 2026-09-02 by
[I049][sokf:issue-049-feature-request-a-contract-cannot-point-at-its-definition].
The `json` block was a hand-written copy of what `validate --json`
emits. Under I049 the contract includes the emitting code's marked
region from source, so there is no copy to leave unbound. What the
block could not reach — that the keys are emitted — is behaviour, and a
behaviour test covers it.

## Summary

`contract-002` declares the keys `superdev validate --json` emits, and no
test compares that list to the binary — the gap
[I035][sokf:issue-035-feature-request-a-contract-does-not-define-its-interface]
closed for the command tree and not for the report it prints. Two keys the binary has been
emitting — `documents` and `schemas` — were never declared, and nothing
reported it.

## Environment

- Version/commit: 0.2.0 / 0e33a51
- Platform: any — the gap is a missing binding, not a runtime failure

## Steps to reproduce

1. `RS_c1` Run `superdev validate --json` and read the top-level keys.
2. `RS_c2` Read the `json:` block of `superdev validate` in
   [contract-002][sokf:contract-002-cli-superdev].
3. `RS_c3` Compare them by hand, which is the only way they have ever been
   compared.

## Expected behaviour

A key the binary emits and the contract does not declare fails a test
naming it, as an undeclared flag or command does — the contract defines
the interface, so a caller can build against the JSON without reading
the code.

## Actual behaviour

The two lists disagreed and every test passed:

```text
emitted:  concepts documents files findings knowledge passed schemas
declared: passed concepts files findings knowledge repaired
```

`documents` and `schemas` are emitted and undeclared; `repaired` is
declared and appears only for `--fix`.

## Root cause (if known)

`crates/app/superdev/src/contract.rs` walks the command tree the
framework builds and compares commands, aliases, positional arguments,
flags and their value types. A command's output shape is not something
the framework knows, so the walk cannot reach it — the same reason the
exit codes needed `contract_exit_codes.rs`, which runs the binary. The
`json` block got neither.

## Proposed fix / workaround

- Fix: bind the declared `json` keys to the keys a real run emits, in
  both directions, the way `contract_exit_codes.rs` binds exit codes by
  running the binary. A key that appears only under a flag — `repaired`
  under `--fix` — needs the run that produces it.
- Workaround: read both lists together when either changes.

## Regression risk

The binding would read the same contract block the CLI drift test reads
and the same binary the exit-code probes run, so it adds no new
mechanism; a key added to the report without its declaration is what it
would catch.

## Comments

Found while framing
[I036][sokf:issue-036-feature-request-validate-prints-warnings-by-default],
which adds `errors` and `warnings` to the block and declares the two
keys that had been missing. That closes the instance and not the gap:
the next undeclared key will stand just as quietly.

Framed into
[I045][sokf:issue-045-feature-request-drift-tests-bind-what-the-contract-declares],
whose criteria 1 to 3 close the gap. The binding stops at the top-level
keys: the per-finding `severity`, `file` and `message` are stated in one
prose sentence rather than declared, so binding them needs the block
restructured first.

<!-- sokf:links -->
[sokf:contract-002-cli-superdev]: /knowledge/contracts/public/active/contract-002-cli-superdev.md
[sokf:issue-035-feature-request-a-contract-does-not-define-its-interface]: /knowledge/issues/done/issue-035-feature-request-a-contract-does-not-define-its-interface.md
[sokf:issue-036-feature-request-validate-prints-warnings-by-default]: /knowledge/issues/done/issue-036-feature-request-validate-prints-warnings-by-default.md
[sokf:issue-045-feature-request-drift-tests-bind-what-the-contract-declares]: /knowledge/issues/wontfix/issue-045-feature-request-drift-tests-bind-what-the-contract-declares.md
[sokf:issue-049-feature-request-a-contract-cannot-point-at-its-definition]: /knowledge/issues/open/issue-049-feature-request-a-contract-cannot-point-at-its-definition.md
