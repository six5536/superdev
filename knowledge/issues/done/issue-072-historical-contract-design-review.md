---
type: Issue
id: issue-072-historical-contract-design-review
title: 'Historical plan: Contract-design review and the binding-surface standard'
description: Migration-derived issue preserving the primary issue relationship for plan-019-contract-design-review.
kind: chore
lifecycle: done
---

# Chore: historical plan: contract-design review and the binding-surface standard

## Summary

Preserve `plan-019-contract-design-review` as queryable workflow history under the canonical issue/plan relationship.

## Context

The plan predates the requirement that every plan implements exactly one issue and had no recoverable primary issue link.

## Behaviour

The migrated plan has one `implements` relationship to this issue, so incoming graph edges expose its history without invented product requirements.

## Scope

Mechanical metadata and required schema sections only; historical intent and product behavior are unchanged.

## Resolution

The historical work is represented by `plan-019-contract-design-review`; this migration-derived issue preserves the required one-issue-per-plan relationship.
