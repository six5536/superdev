---
type: Issue
id: issue-071-historical-code-review-at-the-last-slice
title: 'Historical plan: Integrate runs /code-review once, at the last slice'
description: Migration-derived issue preserving the primary issue relationship for plan-015-code-review-at-the-last-slice.
kind: chore
lifecycle: done
---

# Chore: historical plan: integrate runs /code-review once, at the last slice

## Summary

Preserve `plan-015-code-review-at-the-last-slice` as queryable workflow history under the canonical issue/plan relationship.

## Context

The plan predates the requirement that every plan implements exactly one issue and had no recoverable primary issue link.

## Behaviour

The migrated plan has one `implements` relationship to this issue, so incoming graph edges expose its history without invented product requirements.

## Scope

Mechanical metadata and required schema sections only; historical intent and product behavior are unchanged.

## Resolution

The historical work is represented by `plan-015-code-review-at-the-last-slice`; this migration-derived issue preserves the required one-issue-per-plan relationship.
