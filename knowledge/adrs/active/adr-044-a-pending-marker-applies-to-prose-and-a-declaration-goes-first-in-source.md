---
type: Decision
id: adr-044-a-pending-marker-applies-to-prose-and-a-declaration-goes-first-in-source
title: A pending marker applies to prose, and a declaration goes first in source
description: A definition element cannot run ahead of its source once the definition is materialised from it, so contract-first for a definition is declaration-first in source — CONTRACT-DESIGN writes the field or the path into the marked region with its behaviour unbuilt — and the pending marker narrows to prose promises, where accept still refuses one.
lifecycle: active
links:
  - rel: supersedes
    to: adr-038-a-contract-may-promise-what-is-not-built-yet
    note: The marker survives for prose; a definition element has no place to carry one, because it is the source.
  - rel: references
    to: adr-042-a-contracts-definition-is-materialized-from-source
    note: The decision that makes a definition element and its source the same thing.
  - rel: references
    to: adr-028-the-contract-design-go-ahead-is-an-explicit-interaction
    note: The declaration-only source edit lands under the approval this already requires.
---

# ADR-044: A pending marker applies to prose, and a declaration goes first in source

- Date: 2026-09-02
- Deciders: superdev maintainers

## Context

[ADR-038][sokf:adr-038-a-contract-may-promise-what-is-not-built-yet]
let a contract promise an element ahead of its code, marked `PENDING`,
so a contract could merge days before the code that meets it, with
accept refusing any contract still carrying the marker once the feature
settled. That was contract-first made workable: the definition block
was authored, so an element could be written into it before anything
implemented it.

Under
[ADR-042][sokf:adr-042-a-contracts-definition-is-materialized-from-source]
the definition block is materialised from source. If the flag is not in
the struct, it is not in the include; there is no authored block for a
pending element to be written into. The marker has nowhere to sit for
a definition element, and contract-first needs a different shape.

A prose promise is unchanged. A `MUST` in the Behaviour section for
behaviour not yet built can still run ahead of its code, and the reader
still needs to see that it does.

## Decision

Contract-first for a definition is declaration-first in source.
CONTRACT-DESIGN writes the new element — the field, the argument, the
path, the message — into the marked region of the source, with its
behaviour unbuilt, and the include shows it at once. BUILD implements
behind the declaration. The phase produces a declaration-only source
edit beside its document edits, under the approval
[ADR-028][sokf:adr-028-the-contract-design-go-ahead-is-an-explicit-interaction]
already requires; the diff the user approves now shows the declaration
in the language it will ship in.

The `pending` marker applies to prose alone, and takes one form: the
word `PENDING` in uppercase, beside the statement's modal verb, naming
in parentheses the issue or plan slice that will build it — "the
validator MUST accept a sixth kind, PENDING (I049)". Uppercase binds,
as the RFC 2119 keywords do, so `\bPENDING\b` is what accept greps in
Behaviour and Stability, and accept refuses a settling contract that
still carries one. A definition element carries no marker: a
declaration whose behaviour is unbuilt says so in the language's own
terms — a failing behaviour test, a `todo!()` — and the include shows
the declaration either way.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Pending narrows to prose, as an uppercase `PENDING` naming its slice; declarations go first in source | Contract-first survives in the strongest form — the declaration is in the code before the code is; the marker keeps its one remaining meaning and one greppable form | CONTRACT-DESIGN edits source, which the phase never did before |
| Retire pending entirely | Simplest; a promise not built is the plan's to track | The reader loses the in-document signal that a Behaviour promise is ahead of its code |
| A `sokf:pending` marker in source, carried through the include | The reader sees it in the contract | A comment in code saying "this does not work yet", which a failing test and a `todo!()` already say in the language's own terms |
| Keep an authored block beside the include for pending elements | ADR-038 unchanged | A hand-written copy returns, for exactly the elements most likely to drift |

## Consequences

- Positive: a promised element is visible in the contract the moment it
  is declared, and cannot disagree with the declaration; the marker
  keeps one meaning.
- Negative: CONTRACT-DESIGN's diff includes source; a declaration with
  unbuilt behaviour is on the branch until BUILD reaches it, which is
  what contract-first has always meant.
- Follow-ups: the contract-design skill's commit step names the
  declaration edit; the accept skill's pending gate reads Behaviour and
  Stability only.

<!-- sokf:links -->
[sokf:adr-028-the-contract-design-go-ahead-is-an-explicit-interaction]: /knowledge/adrs/active/adr-028-the-contract-design-go-ahead-is-an-explicit-interaction.md
[sokf:adr-038-a-contract-may-promise-what-is-not-built-yet]: /knowledge/adrs/deprecated/adr-038-a-contract-may-promise-what-is-not-built-yet.md
[sokf:adr-042-a-contracts-definition-is-materialized-from-source]: /knowledge/adrs/active/adr-042-a-contracts-definition-is-materialized-from-source.md
