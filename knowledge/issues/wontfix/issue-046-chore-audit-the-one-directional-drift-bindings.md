---
type: Chore
id: issue-046-chore-audit-the-one-directional-drift-bindings
title: Three drift bindings compare in one direction only, and nothing records whether that is deliberate
description: The config, lock and internal-interface bindings each assert one difference and not its reverse; one of the three is plainly deliberate, one is plainly not, and no comment in the tests says which is which.
lifecycle: wontfix
links:
  - rel: references
    to: issue-049-feature-request-a-contract-is-not-written-to-be-compared
    note: Supersedes this chore; its both-directions criterion covers the three bindings.
  - rel: references
    to: adr-036-a-contract-is-bound-to-its-implementation
    note: The obligation is element for element; a one-directional binding meets half of it.
  - rel: references
    to: issue-045-feature-request-drift-tests-bind-what-the-contract-declares
    note: Found by the framing audit for I045, which put this surface out of its scope.
---

# Chore: audit the one-directional drift bindings

## Won't fix

Superseded 2026-09-02 by
[I049][sokf:issue-049-feature-request-a-contract-is-not-written-to-be-compared].
A binding that runs one way is the direction property failing, not a
separate fault, and I049 requires every element of every contract this
repository owns to be compared in both directions. The audit this chore
asked for is that criterion's work.

## Summary

Three of the drift bindings compare one direction and not the other, and
nothing on file says whether each omission was a decision or an
oversight.
[ADR-036][sokf:adr-036-a-contract-is-bound-to-its-implementation] binds a
contract to its implementation element for element; a binding that
subtracts one way meets half of that. Deciding each one and recording the
reason is small; leaving them undistinguished means a reader cannot tell
a deliberate half-binding from a forgotten one.

## Surfaces

- `crates/lib/superdev-core/tests/contract_files.rs:133` — the config
  binding asserts written keys against declared keys and not the
  reverse. Plausibly deliberate: `contract-004` declares what a deployer
  may supply, and superdev writes only part of it, so a declared key
  nothing writes is not by itself a fault.
- `crates/lib/superdev-core/tests/contract_files.rs:219` — the lock
  binding has the same shape. Plausibly not deliberate: superdev writes
  the whole lock file, so a key `contract-006` declares and nothing
  writes is an outstanding promise (ADR-038) that should fail.
- `crates/lib/superdev-core/tests/contract_interfaces.rs:214` — asserts
  `PENDING` alone, over `checked >= 60` declared items. Plausibly
  deliberate: not every public item in `crates/` belongs to an internal
  contract, so the reverse direction has no defined set to compare
  against.
- No comment in any of the three states which case it is; the two in
  `contract_files.rs` sit beside token and template bindings at lines
  269-308 that do assert both directions.

## Definition of done

- Each of the three bindings carries a comment stating the direction it
  binds and why the other is absent, or gains the missing assertion.
- The lock binding fails when `contract-006` declares a key the writer
  never emits, proved by adding one and watching it fail
  (`cargo nextest run -E 'test(the_written_lock_declares)'` — the exact
  test name to be read off the file).
- `cargo nextest run` passes with no test weakened: the audit adds
  assertions or comments and removes none.

## Comments

Found while framing
[I045][sokf:issue-045-feature-request-drift-tests-bind-what-the-contract-declares]
by auditing all four drift tests for I044's bare-equality fault. This is
a different fault from the one I045 closes — a direction not compared,
rather than a difference reported without its direction — so it was
filed rather than absorbed.

<!-- sokf:links -->
[sokf:adr-036-a-contract-is-bound-to-its-implementation]: /knowledge/adrs/active/adr-036-a-contract-is-bound-to-its-implementation.md
[sokf:issue-045-feature-request-drift-tests-bind-what-the-contract-declares]: /knowledge/issues/wontfix/issue-045-feature-request-drift-tests-bind-what-the-contract-declares.md
[sokf:issue-049-feature-request-a-contract-is-not-written-to-be-compared]: /knowledge/issues/open/issue-049-feature-request-a-contract-is-not-written-to-be-compared.md
