---
type: Decision
id: adr-040-a-warning-is-counted-by-default-and-listed-on-request
title: A warning is counted by default and listed on request
description: validate lists errors and counts warnings; `--warnings` lists them, the CLI and the PostToolUse hook default alike, and `--json` reports the same information as the text output, which means carrying both counts it does not carry today.
lifecycle: active
---

# ADR-040: A warning is counted by default and listed on request

- Date: 2026-09-01
- Deciders: superdev maintainers

## Context

Every run prints its warnings, and the PostToolUse hook fires on every
edit to a governed file, so an agent editing one document reads the whole
repository's advisory findings on every pass. Filing the issues on this
branch surfaced the same five — skills whose frontmatter carries keys
outside the Agent Skills spec — roughly a dozen times without one of them
being acted on.

[ADR-039][sokf:adr-039-a-decidable-finding-is-an-error-and-the-turn-is-the-gate]
moved five findings out of the warning tier, so what remains is exactly
what the repository cannot settle alone: a `rel` whose meaning is the
consumer's, a frontmatter key whose portability depends on where a skill
is published, a length limit that only degrades. None of them is
actionable for the edit in hand, and each is worth knowing about once.

Hiding them outright is the shape that produced the 39 unactioned
findings I012 was filed over. The difference is not how loudly to
complain — that was ADR-039's question — but whether a finding nobody can
act on now belongs in the output of every run.

## Decision

A run lists its errors and counts its warnings. The summary states both
counts, so a reader always knows something is there; `--warnings` lists
them.

The CLI and the PostToolUse hook default alike. One rule holds whoever
ran the command, and CI passes the flag when it wants the whole picture.

`--json` reports the same information as the text output: both counts
always, and the findings the text run listed. It carries no counts today,
so a consumer derives them from `findings` — suppressing a warning there
would lose its count entirely, which is why the counts land first. The
two keys the binary already emits without declaring them, `documents` and
`schemas`, are declared with them.

The default changes what a run shows and never what it decides. Exit
codes do not move, and no finding stops being found.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Count by default, list on request | The noise goes and the fact of it stays; one rule for both surfaces | A reader must know the flag to see the detail |
| Drop warnings from the output entirely | Quietest | The shape that produced 39 unactioned findings; a reader cannot tell there is anything to see |
| Hide in the hook, show in the CLI | Nothing to learn for a human | The same command behaves differently by caller, which is more to explain than it saves |
| A config key rather than a flag | A team decides once | Decides once for a repository where a flag decides per run, and a one-off look still needs a flag |
| Leave `--json` listing everything | Existing consumers keep what they have | Text and JSON would then report different things, which is a worse promise than either default |

## Consequences

- Positive: an agent's every-edit output carries what it can act on, and
  a count of what it cannot.
- Positive: `--json` gains the counts it never had, so a consumer stops
  deriving them from a list that is about to become conditional.
- Negative: a warning is one flag further away, and a reader who has not
  met `--warnings` sees only a number.
- Negative: the `json` keys the CLI contract declares are bound by no
  test, which is how two undeclared keys stood; this decision declares
  them and leaves the binding to its own work.

<!-- sokf:links -->
[sokf:adr-039-a-decidable-finding-is-an-error-and-the-turn-is-the-gate]: /knowledge/adrs/active/adr-039-a-decidable-finding-is-an-error-and-the-turn-is-the-gate.md
