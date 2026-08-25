---
type: Template
id: template-postmortem
title: Postmortem Template
description: Blameless incident write-up — impact, timeline, root cause, and typed action items.
status: stable
---

# Postmortem: <incident title, e.g. "API outage 2026-08-24">

- Date of incident: <YYYY-MM-DD, HH:MM–HH:MM TZ>
- Duration: <detection to resolution>
- Severity: <SEV level or plain-language impact tier>
- Authors: <names>
- Status: draft | reviewed | action items complete

> Blameless: this document names systems and processes, not people.

## Summary

<Three or four sentences: what broke, who was affected and how badly, how it was resolved. Written for someone who reads nothing else.>

## Impact

- <Users/requests/data affected, quantified where possible>
- <Business or downstream impact>

## Timeline

| Time (TZ) | Event |
|-----------|-------|
| HH:MM | <First bad deploy / trigger> |
| HH:MM | <Alert fired / first user report — detection> |
| HH:MM | <Key investigation step or wrong turn> |
| HH:MM | <Mitigation applied> |
| HH:MM | <Full resolution confirmed> |

## Root cause

<The mechanism, not just the trigger: the chain from initial condition to user-visible failure. Include why defenses (tests, review, alerts, rollout) didn't catch it.>

## What went well / what went poorly

- Well: <e.g. alerting fired within 2 minutes>
- Poorly: <e.g. runbook was stale; rollback took 40 minutes>
- Lucky: <where luck, not process, limited the damage>

## Action items

| # | Action | Type | Owner | Due |
|---|--------|------|-------|-----|
| 1 | <Specific, verifiable change> | prevent / detect / mitigate | <name> | <date> |
| 2 | <...> | | | |
