---
type: Template
id: template-pr-description
title: PR Description Template
description: Summary, motivation, grouped changes, test plan, and notes for reviewers.
status: stable
---

# <PR title: imperative, ≤72 chars, e.g. "Add retry logic to sync client">

## Summary

<1–3 bullet points: what this PR does and why. Lead with the user-visible or behavioral change, not the mechanics.>

- <Change 1>
- <Change 2>

## Motivation

<Why this change is needed. Link the issue/ticket if one exists. One short paragraph; skip if the summary already makes it obvious.>

## Changes

<Only for larger PRs — group the diff into a reviewable narrative:>

- <Area/module>: <what changed and why>
- <Area/module>: <what changed and why>

## Test plan

- [ ] <Automated tests added/updated — name them>
- [ ] <Commands run and their results, e.g. `npm test` passes>
- [ ] <Manual verification steps, if any>

## Notes for reviewers

<Anything that makes review easier: suggested reading order, known trade-offs, follow-ups deferred to later PRs. Delete if empty.>

🤖 Generated with [Claude Code](https://claude.com/claude-code)
