---
type: FeatureRequest
id: issue-049-feature-request-a-contract-is-not-written-to-be-compared
title: A contract is required to be compared to its code, and never required to be written so that it can be
description: ADR-036 obliges a project to bind its contract to its implementation element for element and leaves the mechanism to the project, and nothing asks the contract to carry what a comparison needs, so five open issues are one property failing in five places.
lifecycle: open
links:
  - rel: references
    to: adr-036-a-contract-is-bound-to-its-implementation
    note: States the obligation to compare and asks nothing of the contract that would let a comparison be written.
  - rel: references
    to: adr-038-a-contract-may-promise-what-is-not-built-yet
    note: Supplies one of the two directions a difference must name, and nothing supplies the other.
  - rel: references
    to: adr-034-each-kind-defines-in-the-form-its-ecosystem-reads
    note: Stands unchanged; this declares the form each kind already chose where a machine reads it.
  - rel: references
    to: adr-039-a-decidable-finding-is-an-error-and-the-turn-is-the-gate
    note: Draws the line that keeps the code half of the comparison out of the validator's reach.
  - rel: references
    to: issue-043-bug-the-cli-contracts-json-keys-are-bound-by-no-test
    note: The measured instance of the granularity property failing; its criteria are carried here.
  - rel: references
    to: issue-044-bug-a-drift-test-names-the-direction-for-a-command-and-not-for-a-flag
    note: The measured instance of the identity and direction properties failing; its criteria are carried here.
  - rel: references
    to: issue-045-feature-request-drift-tests-bind-what-the-contract-declares
    note: Superseded — its seven criteria are carried here as this repository's application of the standard.
  - rel: references
    to: issue-046-chore-audit-the-one-directional-drift-bindings
    note: Superseded — a one-directional binding is the direction property failing, not a separate fault.
  - rel: references
    to: issue-047-feature-request-a-kinds-definition-form-is-stated-in-prose
    note: Superseded — a form nothing reads is the enumerability property failing.
  - rel: references
    to: issue-048-feature-request-no-step-asks-whether-a-contract-is-bound
    note: Deliberately separate — whether anyone did the work is a different question from whether it can be done.
---

# Feature: a contract is not written to be compared

## Summary

[ADR-036][sokf:adr-036-a-contract-is-bound-to-its-implementation]
obliges a project to bind its contract to its implementation, element
for element in both directions, and leaves the mechanism to the project.
Nothing asks the contract to be written so that the binding can be
written at all. The properties a comparison needs are stated nowhere, so
each fails on its own and is filed on its own — five open issues on this
tracker are one property failing in five places.

## Motivation

A comparison between a contract and its code needs four things. Naming
them once shows that the five issues are one.

**The contract's elements can be listed.** This is the one property
already required — `contract-style` says a contract defines every
element in the structured form its schema declares, and that prose
describes and never defines. What is absent is any way to tell which
form a block is in. Nine of the sixteen kinds name their form in a
section description nothing reads, and `content: code` is satisfied by
any fence whatever its tag, so `contract-008` carries a `text` block its
kind names nowhere and passes
([I047][sokf:issue-047-feature-request-a-kinds-definition-form-is-stated-in-prose]).

**Each element carries a name the code side can match.** Stated nowhere.
A comparison without per-element identity can only compare whole
structures, which is what `contract.rs:403-408` and `mcp.rs:1027-1030`
do — one `assert_eq!` over a `Surface`, printing two struct dumps under
an undirected `DRIFT —`
([I044][sokf:issue-044-bug-a-drift-test-names-the-direction-for-a-command-and-not-for-a-flag]).

**The code's elements are obtainable at the granularity the contract
declares.** Stated nowhere. `contract-002` declares ten top-level JSON
keys; clap introspection cannot reach a command's output, so nothing
compares them. Ten are emitted and ten declared, reconciled once by
plan-023 and kept reconciled by nothing
([I043][sokf:issue-043-bug-the-cli-contracts-json-keys-are-bound-by-no-test]).
Exit codes hit the same wall and were solved by running the binary — one
of four strategies this repository's own tests use between them and no
document records.

**A difference names its element and its direction.** Stated in half.
[ADR-038][sokf:adr-038-a-contract-may-promise-what-is-not-built-yet]
gives a contract ahead of its code the `pending` marker; nothing covers
code ahead of its contract, and three bindings — the config, the lock
and the internal interfaces — assert one difference and never its
reverse
([I045][sokf:issue-045-feature-request-drift-tests-bind-what-the-contract-declares],
[I046][sokf:issue-046-chore-audit-the-one-directional-drift-bindings]).

The cost is measurable twice. Every one of these was found by a person
reading the contract beside the tests, never by a check — and each was
then filed as its own feature, so the same property is being specified
five times in five vocabularies. The obligation has shipped in all
sixteen kind schemas since it was taken, and produced two unbound
contracts in this repository alone.

## Proposed behaviour

A project reading superdev's contract standard finds what makes a
contract comparable, stated once: its elements can be listed, each
carries a name the code side can match, and a difference names the
element and the side that carries it. The standard says which half is
whose. superdev requires the contract's form, because the form is
superdev's; the project supplies the comparison, because only the
project knows its framework, its language and its test runner.

Beside the requirement the project finds the strategies that work,
named and not implemented: introspect the framework, run the interface
and read what it produces, parse the artifact the code writes, and match
declared elements against the source. A contract whose elements no
strategy can reach is written at a granularity its implementation
cannot report, which is a fault in the contract rather than a gap in the
test.

Each kind's definition form is declared where a machine reads it, so a
contract carrying a block in some other form is an error rather than a
silent pass.

superdev's own contracts and comparisons meet the standard, which is
where the five issues above are settled.

## Acceptance criteria

1. [ubiquitous] THE SYSTEM SHALL state, in the contract standard every
   contract-kind schema carries, what makes a contract comparable to its
   implementation: its elements listable, each element named, and a
   difference reported with its element and its direction.
2. [ubiquitous] THE SYSTEM SHALL state, for each of those, whether
   superdev requires it of the contract's form or the project supplies
   it in its own test.
3. [ubiquitous] THE SYSTEM SHALL name the strategies by which a project
   may obtain its implemented surface — introspecting the framework,
   running the interface, parsing the artifact the code writes, and
   matching declared elements against source — and SHALL supply an
   implementation of none.
4. [ubiquitous] THE SYSTEM SHALL declare, in each contract-kind schema,
   the form its definition section takes, where a machine reads it
   rather than in prose.
5. [event] WHEN a contract's definition section carries a fence tag its
   kind does not declare THE SYSTEM SHALL report an error naming the
   declared form and the tag found.
6. [ubiquitous] THE SYSTEM SHALL require each element a contract
   declares to carry a name unique within that contract, so a comparison
   matches element to element rather than structure to structure.
7. [event] WHEN a comparison of this repository's contracts finds a
   difference THE SYSTEM SHALL name the differing element and state
   which of the contract and the implementation carries it.
8. [ubiquitous] THE SYSTEM SHALL compare, in both directions, every
   element of every contract this repository owns, including the
   top-level keys `superdev validate --json` emits and the key a `--fix`
   run adds.
9. [conditional] IF a contract declares an element that no comparison
   reaches THE SYSTEM SHALL fail naming that element, rather than
   passing it uncompared.
10. [state] WHILE a contract element is marked pending THE SYSTEM SHALL
    report the difference as an outstanding promise rather than as a
    defect.
11. [event] WHEN a comparison reports a difference without naming its
    direction THE SYSTEM SHALL fail the check that binds comparison
    reporting.
12. [ubiquitous] THE SYSTEM SHALL leave every contract on file passing
    validation, and SHALL record, for any contract whose fence tag its
    kind does not admit, whether the kind widened or the contract
    changed.

## Alternatives considered

- Fix the five issues separately, as was happening. Rejected: the same
  property was being specified five times in five vocabularies, and each
  instance was found by a person reading rather than by the fix before
  it.
- State the property and supply the comparison too. Rejected: the
  mechanism depends on the project's language, framework and test
  runner, which superdev cannot know — the non-goal in
  `constraints-non-goals`, and ADR-036's own reasoning.
- Leave it at ADR-036's obligation. Rejected: the obligation has shipped
  in all sixteen kind schemas since it was taken, and this repository —
  the one that wrote it — has produced two unbound contracts under it.
- Make the whole property a validator check. Rejected: the code half is
  not decidable from the knowledge tree, so
  [ADR-039][sokf:adr-039-a-decidable-finding-is-an-error-and-the-turn-is-the-gate]
  puts it out of reach by construction. Only the contract's half can be
  checked there, which is what criteria 4 to 6 do.
- Generate the surface from the contract and compare nothing. Rejected:
  generation binds only where the build performs it and is usually
  partial (ADR-036), and superdev governs repositories whose surfaces it
  cannot generate.
- Reduce the number of definition languages so one parser reaches every
  contract. Rejected: a comparison is written in the adopting project's
  own language, and that project already owns the tooling for its own
  ecosystem's form, so
  [ADR-034][sokf:adr-034-each-kind-defines-in-the-form-its-ecosystem-reads]'s
  per-ecosystem choice costs the comparison nothing.

## Scope

- In: the statement of what makes a contract comparable, in the contract
  standard all sixteen kind schemas carry.
- In: the division between what superdev requires of the contract and
  what the project supplies.
- In: the strategies, named as guidance.
- In: the machine-readable declaration of each kind's definition form,
  and the fence-tag check it makes possible.
- In: the requirement that a declared element carry a name.
- In: this repository's own contracts and comparisons meeting the
  standard, which settles I043, I044, I045, I046 and I047.
- Out: supplying a harness, a generator, a comparison or a gate for a
  managed project. The non-goal stands; this sharpens where it falls
  rather than reversing it.
- Out: whether an individual contract names the test that binds it, and
  whether any step asks whether a binding exists, which are
  [I048][sokf:issue-048-feature-request-no-step-asks-whether-a-contract-is-bound]'s.
- Out: the keys inside a `findings` entry, which need `contract-002`'s
  prose sentence restructured before anything can bind them.
- Out: changing any kind's definition form. ADR-034 stands.

## Comments

Framed 2026-09-02 after the five issues were read together rather than
one at a time. Each had been framed on its own mechanism — an unbound
JSON block, a struct dump, a prose form declaration, a one-directional
test — and none stated the property it served. The user's framing is
what unified them: contracts should be written in a clear way so that
the contract and the code can be compared.

I043 and I044 stay open as the defect records and are closed by this
feature's slices. I045, I046 and I047 are superseded, their criteria
carried into the list above.
[I048][sokf:issue-048-feature-request-no-step-asks-whether-a-contract-is-bound]
stays separate on purpose: this feature makes a comparison writable, and
I048 asks whether anyone wrote one.

Two questions are inherited by CONTRACT-DESIGN. The vocabulary for the
two directions is one — a contract ahead of its code has `pending`,
which declares the gap deliberate and forces its own removal, while code
ahead of its contract has no marker, and whether a symmetric affordance
should exist is undecided; the recorded view is that it should not,
because it would mark a state that never merges. The other is the shape
of the form declaration: most of the nine kinds name a set of forms
rather than one, and the three text-format contracts on file carry two
different tags between them, so a single-string declaration cannot admit
all three.

<!-- sokf:links -->
[sokf:adr-034-each-kind-defines-in-the-form-its-ecosystem-reads]: /knowledge/adrs/active/adr-034-each-kind-defines-in-the-form-its-ecosystem-reads.md
[sokf:adr-036-a-contract-is-bound-to-its-implementation]: /knowledge/adrs/active/adr-036-a-contract-is-bound-to-its-implementation.md
[sokf:adr-038-a-contract-may-promise-what-is-not-built-yet]: /knowledge/adrs/active/adr-038-a-contract-may-promise-what-is-not-built-yet.md
[sokf:adr-039-a-decidable-finding-is-an-error-and-the-turn-is-the-gate]: /knowledge/adrs/active/adr-039-a-decidable-finding-is-an-error-and-the-turn-is-the-gate.md
[sokf:issue-043-bug-the-cli-contracts-json-keys-are-bound-by-no-test]: /knowledge/issues/open/issue-043-bug-the-cli-contracts-json-keys-are-bound-by-no-test.md
[sokf:issue-044-bug-a-drift-test-names-the-direction-for-a-command-and-not-for-a-flag]: /knowledge/issues/open/issue-044-bug-a-drift-test-names-the-direction-for-a-command-and-not-for-a-flag.md
[sokf:issue-045-feature-request-drift-tests-bind-what-the-contract-declares]: /knowledge/issues/wontfix/issue-045-feature-request-drift-tests-bind-what-the-contract-declares.md
[sokf:issue-046-chore-audit-the-one-directional-drift-bindings]: /knowledge/issues/wontfix/issue-046-chore-audit-the-one-directional-drift-bindings.md
[sokf:issue-047-feature-request-a-kinds-definition-form-is-stated-in-prose]: /knowledge/issues/wontfix/issue-047-feature-request-a-kinds-definition-form-is-stated-in-prose.md
[sokf:issue-048-feature-request-no-step-asks-whether-a-contract-is-bound]: /knowledge/issues/open/issue-048-feature-request-no-step-asks-whether-a-contract-is-bound.md
