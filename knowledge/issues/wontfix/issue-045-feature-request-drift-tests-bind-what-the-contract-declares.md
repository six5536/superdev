---
type: FeatureRequest
id: issue-045-feature-request-drift-tests-bind-what-the-contract-declares
title: A drift test binds only what its framework can report, and a failure inside a bound element does not name its direction
description: The parts of a contract no introspection reaches — the CLI's `json` block — are compared to nothing, and where a drift test does compare a bound element's contents it prints two structs instead of saying whether the difference is a defect or an outstanding promise.
lifecycle: wontfix
links:
  - rel: references
    to: issue-049-feature-request-a-contract-is-not-written-to-be-compared
    note: Supersedes this issue; the seven criteria below are carried there.
  - rel: references
    to: issue-035-feature-request-a-contract-does-not-define-its-interface
    note: Criteria 4 and 12 were accepted on a binding thinner than the acceptance implied; this closes the two gaps.
  - rel: references
    to: issue-043-bug-the-cli-contracts-json-keys-are-bound-by-no-test
    note: The unbound `json` block is the instance criteria 1 to 3 close.
  - rel: references
    to: issue-044-bug-a-drift-test-names-the-direction-for-a-command-and-not-for-a-flag
    note: The undirected failure is the instance criteria 4 to 7 close.
  - rel: references
    to: adr-036-a-contract-is-bound-to-its-implementation
    note: The obligation this feature makes good on, element for element.
  - rel: references
    to: adr-038-a-contract-may-promise-what-is-not-built-yet
    note: Supplies the two directions a failure must choose between.
---

# Feature: a drift test binds only what its framework can report

## Won't fix

Superseded 2026-09-02 by
[I049][sokf:issue-049-feature-request-a-contract-is-not-written-to-be-compared],
which states the property this issue was one instance of: a contract is
written so that it and the code can be compared. All seven criteria
below are carried into I049 as this repository's application of that
standard, and the open question about the direction vocabulary goes with
them. Nothing here is dropped; it is specified one level up.

## Summary

Two parts of the contract-to-implementation binding do not hold. A facet
no introspection reports — the `json` block `superdev validate` declares
— is compared to nothing at all, and where a drift test does reach
inside a bound element it reports the difference without saying which
way it runs. Anyone who reads a green drift run as "the contract and the
binary agree" is reading more than the tests prove.

## Motivation

The gap is measured, twice over.

`documents` and `schemas` were emitted by `superdev validate --json` for
an unknown period while `contract-002` declared neither. Nothing
reported it; it was found by reading the two lists side by side while
framing
[I036][sokf:issue-036-feature-request-validate-prints-warnings-by-default].
That instance is closed — P023 declared both keys — and the gap that let
it stand is not.

[I035][sokf:issue-035-feature-request-a-contract-does-not-define-its-interface]
criterion 12 requires a drift failure to say whether an element is a
defect or an outstanding promise. It was accepted on
`every_drift_test_names_the_direction_it_failed_in`, which asserts the
wordings appear in each drift test's **file**. Two of the four drift
tests — `contract.rs` and `mcp.rs` — carry the wordings in their
set-comparison halves and then fall back to a bare `assert_eq!` over a
whole struct for what a command or a tool *contains*. Both pass a check
that cannot see the failure path it was written to bind.

## Proposed behaviour

The keys `superdev validate --json` emits and the keys `contract-002`
declares are compared against each other, in both directions, by running
the binary — the way `contract_exit_codes.rs` already binds the exit
codes a framework cannot report. A key emitted and undeclared fails
naming it; a key declared and unemitted fails naming it. The comparison
covers `repaired`, which only a `--fix` run produces.

A difference in what a command contains — its flags, its arguments, its
exit map — and in what an MCP tool contains — its arguments — is
reported the way a difference in the command set or the tool set is
reported today: naming the element, and stating whether the binary
carries what the contract omits or the contract promises what the binary
has yet to build — the two directions
[ADR-038][sokf:adr-038-a-contract-may-promise-what-is-not-built-yet]
distinguishes, computed as two set differences rather than recovered from
one struct comparison.

The check that binds drift-test reporting fails when a drift test
reports a difference without stating its direction, so a comparison that
falls back to bare equality cannot pass it.

Together these make good on
[ADR-036][sokf:adr-036-a-contract-is-bound-to-its-implementation]'s
obligation, which binds a contract to its implementation element for
element and is met today only for the elements a framework happens to
report.

## Acceptance criteria

1. [event] WHEN `superdev validate --json` emits a top-level key that
   `contract-002`'s `json` block does not declare THE SYSTEM SHALL fail a
   test naming that key and stating that the binary emits it undeclared.
2. [event] WHEN `contract-002`'s `json` block declares a top-level key
   that `superdev validate --json` does not emit THE SYSTEM SHALL fail a
   test naming that key and stating that the contract declares it
   unemitted.
3. [ubiquitous] THE SYSTEM SHALL cover the `repaired` key by criteria 1
   and 2, which requires exercising `--fix`.
4. [event] WHEN a command's flags, arguments or exit map differ between
   the binary and `contract-002` THE SYSTEM SHALL fail naming the
   differing element and stating which of the two carries it.
5. [event] WHEN a tool's arguments differ between the served MCP surface
   and `contract-003` THE SYSTEM SHALL fail naming the differing
   argument and stating which of the two carries it.
6. [event] WHEN a drift test reports a difference without stating the
   direction the difference runs in THE SYSTEM SHALL fail the check that
   binds drift-test reporting.
7. [ubiquitous] THE SYSTEM SHALL compare every element `contract.rs` and
   `mcp.rs` compare today, so replacing a whole-struct equality with
   per-element comparisons loses no coverage.

## Alternatives considered

- Bind the `json` block by reading the code that emits it rather than by
  running the binary — a second reader of the same source, and it cannot
  see the key `--fix` adds.
- Add the missing wordings to the two failure paths by hand and leave
  the drift-test check textual — the check would then pass for exactly
  the reason it passes now, and the next bare equality would stand just
  as quietly.
- Leave the direction for the reader to recover from the struct diff —
  which is today's behaviour; the direction is what decides whether the
  fix belongs in the contract or in the code, so making the reader
  derive it puts the work in the wrong place.
- Bind the per-finding keys inside `findings` as well — the contract
  states them in one prose sentence, so this needs the `json` block
  restructured first. Deferred rather than rejected.

## Scope

- In: binding the declared `json` keys of `superdev validate` to a real
  run, both directions, `repaired` included.
- In: direction-named failures for what a command contains in
  `contract.rs` and for what a tool contains in `mcp.rs`.
- In: strengthening `every_drift_test_names_the_direction_it_failed_in`
  so it binds a drift test's failures rather than its file text.
- Out: the keys inside a `findings` entry, which need the contract
  restructured before anything can bind them.
- Out: the one-directional bindings in `contract_files.rs` and
  `contract_interfaces.rs`, filed as
  [I046][sokf:issue-046-chore-audit-the-one-directional-drift-bindings].
- Out: any change to what the binary emits or to what the contract
  declares — the ten keys agree today, and this feature is what keeps
  them agreeing.

## Comments

Framed 2026-09-01 from
[I043][sokf:issue-043-bug-the-cli-contracts-json-keys-are-bound-by-no-test]
and
[I044][sokf:issue-044-bug-a-drift-test-names-the-direction-for-a-command-and-not-for-a-flag],
which record the two defects and the evidence for them.

One question is left open for CONTRACT-DESIGN. The two directions are
not symmetric today: a contract ahead of its code has the `pending`
marker ([ADR-038][sokf:adr-038-a-contract-may-promise-what-is-not-built-yet]),
which declares the gap deliberate and forces its own removal, while code
ahead of its contract has no marker and is reported as a `DEFECT`. Code
first is a legitimate order to work in — build the thing, then write
down what it promises — and the label is a verdict on intent the test
cannot know. The asymmetry may be right, because a contract merges ahead
of its code for days while code merges with its contract or not at all;
what is harder to defend is the wording. The criteria above therefore
require a failure to state **which side carries the element** and leave
the vocabulary to the ADR, rather than writing `DEFECT` into the
acceptance the way I035 criterion 12 did.

Four scope decisions were taken at framing. The JSON binding stops at
the top-level keys, because reaching inside `findings` needs a contract
change this feature does not otherwise need. `mcp.rs` joins `contract.rs`
in criterion 5: an audit at framing found the identical bare-equality
fallback there, unfiled. The weak drift-test check is in scope, because
it is what let the fault reach two files. The one-directional bindings
found by the same audit are out, as a different fault on an unmeasured
surface.

<!-- sokf:links -->
[sokf:adr-036-a-contract-is-bound-to-its-implementation]: /knowledge/adrs/active/adr-036-a-contract-is-bound-to-its-implementation.md
[sokf:adr-038-a-contract-may-promise-what-is-not-built-yet]: /knowledge/adrs/active/adr-038-a-contract-may-promise-what-is-not-built-yet.md
[sokf:issue-035-feature-request-a-contract-does-not-define-its-interface]: /knowledge/issues/done/issue-035-feature-request-a-contract-does-not-define-its-interface.md
[sokf:issue-036-feature-request-validate-prints-warnings-by-default]: /knowledge/issues/done/issue-036-feature-request-validate-prints-warnings-by-default.md
[sokf:issue-043-bug-the-cli-contracts-json-keys-are-bound-by-no-test]: /knowledge/issues/open/issue-043-bug-the-cli-contracts-json-keys-are-bound-by-no-test.md
[sokf:issue-044-bug-a-drift-test-names-the-direction-for-a-command-and-not-for-a-flag]: /knowledge/issues/open/issue-044-bug-a-drift-test-names-the-direction-for-a-command-and-not-for-a-flag.md
[sokf:issue-046-chore-audit-the-one-directional-drift-bindings]: /knowledge/issues/wontfix/issue-046-chore-audit-the-one-directional-drift-bindings.md
[sokf:issue-049-feature-request-a-contract-is-not-written-to-be-compared]: /knowledge/issues/open/issue-049-feature-request-a-contract-is-not-written-to-be-compared.md
