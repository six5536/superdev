---
type: FeatureRequest
id: issue-048-feature-request-no-step-asks-whether-a-contract-is-bound
title: Every contract-kind schema requires a binding to the implementation, and no step ever asks whether one exists
description: The obligation ships in all sixteen kind schemas and nothing checks it; the two unbound contracts found so far were both found by a human reading, and an agent step could ask the question at the reliability a judgement carries.
lifecycle: wontfix
links:
  - rel: references
    to: issue-049-feature-request-a-contract-cannot-point-at-its-definition
    note: Folds this in — a materialised definition is bound by construction, and the agent's question narrows to whether the region is the whole surface.
  - rel: references
    to: adr-036-a-contract-is-bound-to-its-implementation
    note: States the obligation this would ask about; the mechanism stays the project's.
  - rel: references
    to: adr-039-a-decidable-finding-is-an-error-and-the-turn-is-the-gate
    note: Draws the line this sits on the far side of — not decidable from the tree, so a judgement rather than a gate.
  - rel: references
    to: issue-038-bug-the-template-format-contract-is-bound-by-no-drift-test
    note: The measured instance — an unbound contract that stood until someone happened to read for it.
---

# Feature: no step asks whether a contract is bound

## Won't fix

Folded 2026-09-02 into
[I049][sokf:issue-049-feature-request-a-contract-cannot-point-at-its-definition].
Once a contract's definition is materialised from source, whether it is
bound is decidable: the include is current or the run fails. This
issue's question narrows to the one thing that is not decidable — is
the marked region the whole promised surface, and is the prose complete
for what the shape cannot express — and that lands in I049's
judgement step at integration, failure paths included.

## Summary

Every contract-kind schema obliges the project to bind its contract to
its implementation
([ADR-036][sokf:adr-036-a-contract-is-bound-to-its-implementation]), and
no phase of the workflow ever asks whether that happened. The question is not decidable from the knowledge tree, so no
validator can carry it, but it is exactly the shape of question an agent
answers well — and no skill asks it.

## Motivation

Two contracts have been found unbound, and both were found by accident.

[I038][sokf:issue-038-bug-the-template-format-contract-is-bound-by-no-drift-test]
— `contract-008` enumerated five substitution tokens and the shipped
template set, and nothing compared either to `templates.rs`. It stood
until a human read I035's criterion 4 against the test suite, and was
closed by plan-021 slice 12.
[I043][sokf:issue-043-bug-the-cli-contracts-json-keys-are-bound-by-no-test]
— the CLI contract's `json` block, found the same way while framing
I036.

Neither was caught by a check, because there is none. The obligation
ships to every project that adopts superdev, in all sixteen kind
schemas, with nothing anywhere asking after it. A project's first
unbound contract will be found the same way superdev's two were, if it
is found at all.

## Proposed behaviour

At the points the workflow already reads a contract against the code, a
step asks whether anything binds it — a test, or a generated artifact
proved current — and reports the contracts where it finds nothing,
naming what it looked for. The report is a judgement, not a gate: it
neither blocks a merge nor joins the validator's findings, because an
agent reading a test suite is right most of the time and not all of it,
and a check that accuses falsely is a check people learn to skip.

The step ships in the pack, so a project adopting superdev inherits the
question along with the obligation.

## Acceptance criteria

1. [event] WHEN a feature is accepted THE SYSTEM SHALL report each
   contract the feature touched that no test and no generated artifact
   appears to bind.
2. [ubiquitous] THE SYSTEM SHALL name, for each contract it reports,
   what it searched and what it did not find, so a reader confirms or
   dismisses the report without repeating the search.
3. [ubiquitous] THE SYSTEM SHALL present the report as a judgement that
   blocks nothing, and SHALL NOT record it as a validator finding.
4. [state] WHILE a contract element is marked pending THE SYSTEM SHALL
   count the reverse binding ADR-038 requires as that element's binding,
   rather than reporting it as unbound.
5. [ubiquitous] THE SYSTEM SHALL ship the step in the pack, so a project
   adopting superdev inherits it with the schemas that state the
   obligation.
6. [conditional] IF the feature touched no contract THE SYSTEM SHALL say
   so and report nothing further, so an empty report is distinguishable
   from a step that did not run.
7. [conditional] IF the step cannot read the project's tests THE SYSTEM
   SHALL report what it could not read and SHALL NOT report the
   contracts it could not judge as unbound.

## Alternatives considered

- A validator check — the binding lives in the project's test suite, in
  a language superdev does not read, so the question is not decidable
  from the knowledge tree and
  [ADR-039][sokf:adr-039-a-decidable-finding-is-an-error-and-the-turn-is-the-gate]
  puts it out of the validator's reach by construction.
- A line in `definition-of-done` and nothing else — which is what exists
  in effect today, one level of indirection away, and it caught neither
  I038 nor I043.
- Have the contract declare what binds it, so the validator can check
  that the named file exists. This converts part of an undecidable
  question into a decidable one and is the strongest alternative; against
  it, a file existing is weak evidence that it binds anything, and the
  declaration is one more thing to keep current. Worth weighing at
  CONTRACT-DESIGN rather than dismissing here.
- Do nothing, on the ground that the binding is the project's
  responsibility — true of the mechanism (`constraints-non-goals`), and
  not of the question, which costs a paragraph in a skill.

## Scope

- In: the step and the shape of its report. Criterion 1 puts it where a
  feature is accepted; whether `/maintain` runs the same step over every
  contract on a cadence is CONTRACT-DESIGN's to add.
- In: shipping it in the pack, so an adopting project gets it.
- Out: supplying the binding itself — no harness, no generator, no drift
  test, per the non-goal in `constraints-non-goals`.
- Out: any hard gate, exit code or validator finding.

## Comments

Raised 2026-09-02 while recording the non-goal above. The wording first
committed there — that an unbound contract is "undetectable" — read the
validator as the whole product; superdev's skills are agent-run and are
just as much the product. Correcting that is what surfaced this: the
question is unavailable to a gate and entirely available to a judgement.

<!-- sokf:links -->
[sokf:adr-036-a-contract-is-bound-to-its-implementation]: /knowledge/adrs/active/adr-036-a-contract-is-bound-to-its-implementation.md
[sokf:adr-039-a-decidable-finding-is-an-error-and-the-turn-is-the-gate]: /knowledge/adrs/active/adr-039-a-decidable-finding-is-an-error-and-the-turn-is-the-gate.md
[sokf:issue-038-bug-the-template-format-contract-is-bound-by-no-drift-test]: /knowledge/issues/done/issue-038-bug-the-template-format-contract-is-bound-by-no-drift-test.md
[sokf:issue-043-bug-the-cli-contracts-json-keys-are-bound-by-no-test]: /knowledge/issues/wontfix/issue-043-bug-the-cli-contracts-json-keys-are-bound-by-no-test.md
[sokf:issue-049-feature-request-a-contract-cannot-point-at-its-definition]: /knowledge/issues/open/issue-049-feature-request-a-contract-cannot-point-at-its-definition.md
