---
type: Issue
id: issue-053-item-only-pattern-admits-a-nested-item
title: The item-only-pattern check admits a nested item, and three documents and the finding message say it does not
description: A rule declaring `nested` makes a nested item's lines part of an item, so `item-only-pattern` reports no modal verb there, while contract-010, the doc comment it materialises and the grammar file all say the pattern binds only inside a top-level item.
kind: bug
lifecycle: open
links:
  - rel: references
    to: issue-052-the-workflow-carries-more-process-than-it-needs
    note: Found at acceptance of I052.
---

# Bug: item-only-pattern admits a nested item, and three documents say it does not

## Summary

A schema author reads the `item-only-pattern` vocabulary, sees that a
match outside a top-level item is an error, and expects the validator to
report a nested criterion carrying `SHALL`. The validator accepts it.
Three statements of the rule and the finding message all describe the
behaviour the validator had before nested items existed.

## Context

`Items::read` reads a section's items to the depth the rule's `nested`
chain declares, and `check_item_bounds` collects the lines of every item
at every level into the set it calls inside. A nested item's lines are
therefore inside an item whenever the rule declares a level for them.
Where the rule declares none, the depth is zero, the nested bullet's
lines are dropped from every item, and a modal verb on one is reported.
The two probes differ only in the `nested` declaration.

Four places still state the older rule.

- `knowledge/contracts/internal/active/contract-010-interface-document-schemas.md`
  says under "Item keys and bounds" that `item-only-pattern` "is a regex
  that matches only inside a **top-level** item", and its
  `P_item-only-outside` promise reports a match on "a body line outside
  every item".
- The doc comment on `item_only_pattern` in
  `crates/lib/superdev-core/src/validate/schema/document.rs` reads "The
  pattern that may match only inside a top-level item". The include block
  materialises that comment into contract-010's Definition, so the
  contract carries the claim twice.
- `crates/lib/superdev-core/src/validate/schema/grammar.yaml` lists "a
  nested item" among the body lines whose match is an error.
- The finding `check_item_bounds` pushes reads "matches outside a
  top-level item".

The doc comment on `check_item_bounds` itself already describes the
current behaviour: a match may sit inside an item "at any declared
level". The vocabulary the author reads does not. Acceptance of
[I052][sokf:issue-052-the-workflow-carries-more-process-than-it-needs]
found the divergence.

## Behaviour

The contract, the vocabulary, the grammar file and the finding message
state one rule, and the validator keeps it. Contracts bind, so
contract-010 decides which rule that is. Either the "Item keys and
bounds" paragraph and `P_item-only-outside` widen to accept a match
inside an item at any declared level, and the doc comment, the grammar
file and the message follow; or the check narrows to the top level, and
every nested criterion on file carrying a modal verb becomes an error.

The finding message names the wrong boundary under either outcome. The
contract schema declares one nested level, so a bullet below a criterion
sits past the deepest declared level, its lines are dropped from every
item, and a modal verb on it is reported as matching "outside a
top-level item" — while the author looks at a bullet plainly inside one.

`pack/knowledge/schemas/contract.md` and its owned copy at
`knowledge/schemas/contract.md` state, in the contract-style rules, that
"A bullet below a criterion binds nothing". That statement is false. A
bullet below a criterion carrying a modal verb fails validation, and the
message sends the author to the top level rather than to the depth the
schema declares.

## Scope

The four statements of the rule, the finding message, and the
contract-style rule in both copies of `schema-contract`.

- In: contract-010's "Item keys and bounds" paragraph and
  `P_item-only-outside`.
- In: the `item_only_pattern` doc comment, which reaches contract-010's
  Definition through the include.
- In: the `item-only-pattern` entry in `grammar.yaml`.
- In: the wording of the finding, which must name the depth the schema
  declares.
- In: the "A bullet below a criterion binds nothing" rule in
  `knowledge/schemas/contract.md` and `pack/knowledge/schemas/contract.md`.
- Out: the nested-item machinery itself; the depth-limited reading is
  what ADR-051 decided.
- Out: whether a criterion should carry a modal verb at all.

<!-- sokf:links -->
[sokf:issue-052-the-workflow-carries-more-process-than-it-needs]: /knowledge/issues/done/issue-052-the-workflow-carries-more-process-than-it-needs.md
