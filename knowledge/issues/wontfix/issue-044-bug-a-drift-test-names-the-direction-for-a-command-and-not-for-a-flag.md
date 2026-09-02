---
type: BugReport
id: issue-044-bug-a-drift-test-names-the-direction-for-a-command-and-not-for-a-flag
title: A drift failure names its direction for a whole command and not for a flag inside one
description: I035 criterion 12 requires a drift failure to say whether the element is a defect or an outstanding promise; the CLI test does that for a command and falls back to a bare equality assertion for a flag, argument or exit map, and the test that checks the wordings only checks they appear in the file.
lifecycle: wontfix
links:
  - rel: references
    to: issue-049-feature-request-a-contract-cannot-point-at-its-definition
    note: Dissolves this — the copy the test compared no longer exists.
  - rel: references
    to: issue-035-feature-request-a-contract-does-not-define-its-interface
    note: Criterion 12 is satisfied for a command and not for what a command contains.
---

# Bug: a drift failure names its direction for a command and not for a flag

## Won't fix

Dissolved 2026-09-02 by
[I049][sokf:issue-049-feature-request-a-contract-cannot-point-at-its-definition].
The failure compared a hand-written copy of the clap tree to the tree
itself. Under I049 the copy is materialised from source and cannot
differ from it, so `contract.rs`'s comparison is deleted and there is
no failure left to name a direction.

## Summary

A drift failure is meant to say which way the difference runs: an element
the binary carries and the contract omits is a `DEFECT`, one the contract
promises and the binary lacks is `PENDING`
([ADR-036][sokf:adr-036-a-contract-is-bound-to-its-implementation],
[ADR-038][sokf:adr-038-a-contract-may-promise-what-is-not-built-yet]).
The CLI test does that for a whole command. For a flag, an argument or an
exit map *inside* a command it compares two structs and prints both, so
the reader works the direction out from the diff.

## Environment

- Version/commit: 0.2.0 / 0e33a51
- Platform: any

## Steps to reproduce

1. `RS_c1` Add a flag to `superdev validate`'s `flags:` block in
   [contract-002][sokf:contract-002-cli-superdev] that the binary does
   not offer.
2. `RS_c2` Run `cargo nextest run -E 'test(cli_surface_matches_the_contract)'`.

## Expected behaviour

1. `EX_c1` [ubiquitous] `PENDING — the contract promises …` naming the flag, as adding an
   undeclared *command* to the contract reports today.

## Actual behaviour

```text
DRIFT — `superdev validate` differs between the binary and its contract
  left:  Surface { …, flags: {"--doc", "--fix", "--json", "--knowledge", "--repo-root"} }
  right: Surface { …, flags: {"--doc", "--fix", "--json", "--knowledge", "--repo-root", "--warnings"} }
```

The direction is recoverable from the two lines, and is not stated.

## Root cause (if known)

`crates/app/superdev/src/contract.rs` compares the command *set* in both
directions and names each — lines 393 and 401 carry the `DEFECT` and
`PENDING` wordings. The per-command comparison at line 406 is a single
`assert_eq!` over the whole `Surface`, so everything a command contains
shares one undirected message.

`every_drift_test_names_the_direction_it_failed_in` in
`normative_shapes.rs` asserts the wordings appear in each drift-test
file. That is true of `contract.rs` because the command-set halves carry
them, so the file passes while one of its failure paths does not use
either. The check binds a file's vocabulary, not its every failure.

## Proposed fix / workaround

- Fix: compare a command's flags, arguments and exit map as sets, the way
  the command list is compared, and report each difference with the
  direction it runs in.
- Fix: strengthen the wording check so it binds the failure paths rather
  than the file — otherwise the same gap can reopen anywhere a drift test
  falls back to a bare equality assertion.
- Workaround: read the two structs.

## Regression risk

The change is inside one test's reporting, so nothing in the product
moves; the risk is a weaker comparison replacing a whole-struct equality,
which the existing mutations for the CLI surface would catch.

## Comments

Found while framing
[I036][sokf:issue-036-feature-request-validate-prints-warnings-by-default],
whose contract change adds `--warnings` before the code offers it — the
ordinary ADR-038 window, which is exactly when the direction matters
most.

[I035][sokf:issue-035-feature-request-a-contract-does-not-define-its-interface]
criterion 12 was accepted on the strength of the wording check, which is
thinner than the acceptance implied: it proves the words are in the file,
not that every failure uses one.

Framed into
[I045][sokf:issue-045-feature-request-drift-tests-bind-what-the-contract-declares],
whose criteria 4 to 7 close this. An audit at framing found the same
bare-equality fallback in `crates/lib/superdev-core/src/sokf/mcp.rs:1027`,
where a tool's arguments are compared as one struct: the fault reaches
two files, not one, and I045 covers both.

<!-- sokf:links -->
[sokf:adr-036-a-contract-is-bound-to-its-implementation]: /knowledge/adrs/deprecated/adr-036-a-contract-is-bound-to-its-implementation.md
[sokf:adr-038-a-contract-may-promise-what-is-not-built-yet]: /knowledge/adrs/deprecated/adr-038-a-contract-may-promise-what-is-not-built-yet.md
[sokf:contract-002-cli-superdev]: /knowledge/contracts/public/active/contract-002-cli-superdev.md
[sokf:issue-035-feature-request-a-contract-does-not-define-its-interface]: /knowledge/issues/done/issue-035-feature-request-a-contract-does-not-define-its-interface.md
[sokf:issue-036-feature-request-validate-prints-warnings-by-default]: /knowledge/issues/done/issue-036-feature-request-validate-prints-warnings-by-default.md
[sokf:issue-045-feature-request-drift-tests-bind-what-the-contract-declares]: /knowledge/issues/wontfix/issue-045-feature-request-drift-tests-bind-what-the-contract-declares.md
[sokf:issue-049-feature-request-a-contract-cannot-point-at-its-definition]: /knowledge/issues/framed/issue-049-feature-request-a-contract-cannot-point-at-its-definition.md
