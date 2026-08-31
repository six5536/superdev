---
type: Decision
id: adr-018-loop-in-the-skill-enforcement-in-the-hook
title: The Unattended Loop Is a Skill, Enforced by a Hook That Never Parses a Plan
description: The loop over feature-plan, build and integrate lives in a knowledge-carried skill, and a managed Stop hook keeps the turn going by reading only the run state — so the slice format stays pack content and a repo without the hook still gets the behaviour.
lifecycle: active
---

# ADR-018: The Unattended Loop Is a Skill, Enforced by a Hook That Never Parses a Plan

- Date: 2026-08-31
- Deciders: superdev maintainers

## Context

Issue I024 asks the workflow to deliver a feature plan unattended: a loop
over feature-plan, build and integrate that does not stop at turn
boundaries. The workflow phases are knowledge-carried skills — prose that
works in any harness — while turn boundaries are a harness mechanism:
in Claude Code, only a Stop hook can refuse to let a turn end. A loop
carried by prose alone depends on the model staying disciplined over
dozens of boundaries; a loop carried by the binary alone would need the
binary to understand the feature-plan format, which is pack content and
changes in content releases, not binary releases.

## Decision

We will carry the loop in a knowledge-carried skill,
`execute-feature-plan`, and enforce it with a managed Stop hook. The
skill reads the plan, decides each next step and records it in the run
state; the hook reads only the run state — owner, next step, counter —
and decides only whether the turn may end. The hook never opens a plan.

## Options considered

| Option | Pros | Cons |
|--------|------|------|
| Skill carries the loop, hook enforces it | Works without the hook; enforced with it; plan format stays pack content | Two mechanisms to keep aligned |
| Skill alone | One mechanism, no binary change | A model that drifts ends the run silently; nothing enforces the loop |
| Hook parses the plan and chooses the next slice | One authority over what runs next | Ties the feature-plan format to a binary release; the format stops being content |
| The harness's workflow/orchestration tooling | No superdev code at all | Cannot put a question to the user mid-run and resume from a durable record |

## Consequences

- Positive: the feature-plan format changes in a pack release, with no
  binary release.
- Positive: a repo without Claude Code still gets the loop's behaviour
  from the skill; a repo with it gets the behaviour whether or not the
  model stays disciplined.
- Negative: the run state is written by the model through the run verbs,
  so the hook trusts what the skill recorded; the watchdog counter bounds
  the damage of a wrong entry.
- Follow-ups: the run state interface and its verbs are ADR-019; the
  skill and hook ship together with the knowledge capability.
