---
type: Decision
id: adr-019-run-state-is-a-session-owned-file-behind-cli-verbs
title: Run State Is a Session-Owned File Behind CLI Verbs, and the Hook Owns the Counter
description: An unattended run is armed by .superdev/cache/run.toml, created exclusively by superdev run begin and owned by one session; the Stop hook body is superdev hook run, and the hook alone increments the watchdog counter, capped at ten continues without progress.
lifecycle: active
---

# ADR-019: Run State Is a Session-Owned File Behind CLI Verbs, and the Hook Owns the Counter

- Status: accepted
- Date: 2026-08-31
- Deciders: superdev maintainers

## Context

ADR-018 splits the unattended loop between a skill and a Stop hook, which
must share state: whether a run is active, who owns it, what happens
next, and whether it is still progressing. `.claude/settings.json` is
repo-scoped, so the hook fires in every session in the repo; a run
started in one session must not hold the others open, and two runs must
not race on one working tree. A watchdog must survive a model that has
stopped behaving — a counter the skill incremented could be reset by the
very loop it is meant to bound.

## Decision

We will keep the run state in `.superdev/cache/run.toml`, written only
through CLI verbs: `superdev run begin` creates it exclusively and
refuses when one exists, `superdev run advance` records a step forward,
and `superdev run end` removes it. The state names its owning session,
and the hook ignores every other session's turns. The hook body is
`superdev hook run`, beside `hook validate` in the hook namespace. The
hook alone increments the continue counter; `advance` resets it; at ten
continues without an advance the hook lets the run die.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| CLI verbs over an exclusive, session-owned file; hook owns the counter | Exclusive creation refuses a second run; a stalled model cannot keep its own loop alive | Four new verbs on a promised surface |
| Skills hand-write the TOML | No new verbs | Forfeits exclusive creation, so two runs race; hand-written TOML drifts |
| One run state per repo, no owner | Simpler file | A run in one session holds every session in the repo open |
| The skill increments the counter | Hook stays read-only | A model that stopped progressing can keep resetting its own watchdog |
| Hook body at `superdev run hook` | Everything the run facility owns under one verb | Splits hook plumbing across two namespaces; `hook validate` already fixes the convention |

## Consequences

- Positive: a second `begin` in the same working tree is refused, naming
  the owner and how to clear it; two git worktrees have separate caches
  and run in parallel naturally.
- Positive: a run that stops progressing dies within ten turn boundaries.
- Negative: the cap is a fixed constant; a legitimate step that crosses
  more than ten boundaries without an `advance` dies with it. The skill's
  discipline — advance at every real step — is what keeps that from
  happening.
- Negative: `superdev run` and `hook run` join the CLI surface and carry
  its stability promise once released.
- Follow-ups: the state file's shape and the verbs' behaviour are fixed
  in the run-state interface contract.
