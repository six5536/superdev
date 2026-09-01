---
type: BugReport
id: issue-038-bug-the-template-format-contract-is-bound-by-no-drift-test
title: The template format contract binds two enumerable surfaces and no test compares either to the code
description: I035 criterion 4 demands a drift test for every contract whose interface this repository implements; contract-008 enumerates the five substitution tokens and the shipped template set, and nothing compared either to `templates.rs`; bound in both directions by plan-021 slice 12.
lifecycle: done
links:
  - rel: references
    to: issue-035-feature-request-a-contract-does-not-define-its-interface
    note: The gap acceptance found against criterion 4 — the one contract left unbound.
  - rel: references
    to: contract-008-text-format-template
    note: The contract with no drift test.
---

# Bug: the template format contract is bound by no drift test

## Resolved

Bound by plan-021 slice 12, in `contract_files.rs`, in both directions on
both surfaces. The token vocabulary is read out of `templates.rs` as
source text rather than listed in the test, so a sixth `TOKEN_` constant
is caught — a list written in the test would only ever agree with itself.
The shipped set is read from `shipped()` at runtime. Each of the four
directions was driven by mutation: an undeclared token and an
undocumented template report `DEFECT`, an unimplemented token and an
unshipped template report `PENDING`.

## Summary

Eight of the nine active contracts gained a drift test in P021; the
[template format contract][sokf:contract-008-text-format-template] did
not. It enumerates the five substitution tokens and one section per
shipped template, and nothing compares either list to the code that
implements it, so both can drift silently — against criterion 4 of
[I035][sokf:issue-035-feature-request-a-contract-does-not-define-its-interface],
which demands a drift test for every contract whose interface this
repository implements.

## Environment

- Version/commit: 0.2.0 / 19ac275 (`main`)
- Platform: any — the gap is a missing test, not a runtime failure

## Steps to reproduce

1. Add a sixth token to `crates/lib/superdev-core/src/templates.rs`
   beside `TOKEN_PASCAL`, and use it in `substitute`.
2. Run `npm test`.
3. Observe the suite passes with the contract still naming five tokens.
4. Add a third template to `SHIPPED` in the same file.
5. Run `npm test` again, and observe the suite passes with the contract
   still carrying two `### Template:` sections.

## Expected behaviour

A test fails naming the difference and its direction, as it does for
the other eight contracts: a token or a template the binary carries and
the contract does not declare reports as a `DEFECT`, and one the
contract declares and the binary lacks reports as `PENDING`.

## Actual behaviour

No test reads the contract. The bindings on file cover eight contracts
and name no template:

```text
crates/app/superdev/src/contract.rs           contract-002
crates/app/superdev/tests/contract_exit_codes.rs  contract-002
crates/lib/superdev-core/src/sokf/mcp.rs      contract-003
crates/lib/superdev-core/tests/contract_files.rs  contract-004, 005, 006
crates/lib/superdev-core/tests/contract_interfaces.rs  contract-007, 009, 010
```

The two lists agree today — five `TOKEN_*` constants against the five
in the contract's Shape block, and `SHIPPED`'s two entries against the
two `### Template:` sections — so nothing is currently wrong, and
nothing keeps it that way.

## Root cause (if known)

P021 slice 5 bound the file-format contracts through their readers:
`contract_files.rs` parses each declared block with the real manifest,
pack and lock types. The template format has no reader to parse a
declared block with — its surface is a token vocabulary and a shipped
set, both of which live in
`crates/lib/superdev-core/src/templates.rs:23-36` and `:113` as Rust
constants rather than in a parsed file. The slice bound the three
contracts its chosen mechanism reached and did not reach for a second
mechanism, so `contract-008` was left out. Code review 006 finding 6
caught the same slice claiming four contracts and delivering two, and
the fix added the lock without revisiting the template.

## Proposed fix / workaround

- Fix: bind the token vocabulary by comparing the `TOKEN_*` constants to
  the tokens the contract's Shape block names, in both directions.
- Fix: bind the shipped set by comparing `shipped()` to the contract's
  `### Template:` headings, in both directions, so backporting a
  template without documenting it fails.
- Workaround: none — the drift is invisible until someone reads both.

## Regression risk

The binding reads `templates.rs` and `contract-008`, the same two
surfaces `template-backport` writes; a test that compares them would
catch a template added to the binary without its contract section,
which is the drift the CLI contract had already suffered once in this
feature.

<!-- sokf:links -->
[sokf:contract-008-text-format-template]: /knowledge/contracts/public/active/contract-008-text-format-template.md
[sokf:issue-035-feature-request-a-contract-does-not-define-its-interface]: /knowledge/issues/done/issue-035-feature-request-a-contract-does-not-define-its-interface.md
