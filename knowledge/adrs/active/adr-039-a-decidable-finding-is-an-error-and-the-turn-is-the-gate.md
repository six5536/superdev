---
type: Decision
id: adr-039-a-decidable-finding-is-an-error-and-the-turn-is-the-gate
title: A repository-decidable finding is an error, and the turn is where the gate closes
description: The five findings the repository alone can settle become errors with no warning tier; the edit-time hook stops judging the two that span files, and the Stop hook refuses to end a turn while the knowledge carries an error, under a cap and failing open.
lifecycle: active
---

# ADR-039: A decidable finding is an error, and the turn is the gate

- Date: 2026-09-01
- Deciders: superdev maintainers

## Context

The validator warns on five findings the repository alone can settle: a
broken body link, a missing `resource`, a missing `sources[].resource`,
an index entry naming a file that is not there, and a footnote label
joining no source. Nothing fails on them, so nobody reads them: the
canonical knowledge carried 39 unactioned before anyone looked (I012).

SPEC §11 tells consumers to be permissive — never reject knowledge over
a broken link. That binds a reader displaying knowledge. It has been
read as binding a validator too, which is a different job: a validator
that never fails is not permissive, it is ignored.

Two of the five are what ordinary work in progress looks like. A link is
written before the file it points at lands, and an index entry before
the concept it lists. Making them fatal at edit time would block an
agent several edits before the state it is working towards, so a plan
citing the concepts its own slices will add could not be written at all
until every target existed.

A third tier between warning and error was rejected at framing: it is
more vocabulary for the same decision, and nothing would agree which
findings deserved it. The choice is not how loudly to complain. It is
when the gate closes.

## Decision

The five are errors. No tier is added and none of them stays a warning,
because a warning is what produced the 39.

`superdev validate` fails on all five, always. The three that a single
file settles — a missing `resource`, a missing `sources[].resource`, an
unjoined footnote — are also errors at edit time, as they are today.

The two that only the whole tree settles — a broken body link, an index
entry naming a missing file — are not judged by the PostToolUse hook,
which is handed one edited file and cannot see whether the target
arrives in the next edit.

The Stop hook refuses to end a turn while `superdev validate` reports
any error, and names them. The turn, not the edit, is the moment a
document is claimed to be finished, so it is the moment the gate closes.
An agent may write a forward reference and resolve it three edits later;
it may not finish having left one.

Two properties bound the hold. It fails open: knowledge that cannot be
read or checked lets the turn end, because a Stop hook that fails closed
holds every session in the repository open. And it is capped: after a
fixed number of holds in one session it reports and lets the turn end,
so an error the agent cannot resolve stalls nothing.

The permissiveness rule keeps its scope and gains its limit: it binds a
consumer of knowledge, never the validator of a repository.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Error everywhere, gated at the turn | One severity; nothing is ignored; a forward reference stays writable | The Stop hook gains a job it did not have, and a cap to bound it |
| Error everywhere, gated at every edit | Simplest rule, no new machinery, told at once | A document referencing what a later step creates cannot be written until its targets exist |
| Error in `validate`, silent in both hooks | No hook work; CI is the backstop | In a long session CI is far away, and the finding is found by someone else later |
| Keep them warnings | No work at all | The 39 unactioned findings are the measure of it |
| A third severity between warning and error | Names the middle case | More vocabulary for the same decision, and nothing would agree what belongs in it |

## Consequences

- Positive: a dangling link cannot survive a turn, and no finding
  depends on a human noticing it.
- Positive: the two work-in-progress classes stay writable mid-task,
  which is what made a fatal edit-time check unacceptable.
- Negative: the Stop hook now runs the validator once per turn end, and
  carries a cap and a fail-open path that must both be proved.
- Negative: SPEC §10 and §11 change, so every repository the format
  governs is held to the stricter reading.
- Follow-ups: the run-state contract carries the hold count, the CLI
  contract states both hooks' new behaviour, and I036 may then hide the
  three warnings that remain without hiding anything decidable.
