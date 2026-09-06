---
type: Issue
id: issue-068-historical-filing-by-lifecycle
title: 'Historical plan: Documents are filed by lifecycle'
description: Migration-derived issue preserving the primary issue relationship for plan-011-filing-by-lifecycle.
kind: chore
lifecycle: done
---

# Chore: historical plan: documents are filed by lifecycle

## Summary

Preserve `plan-011-filing-by-lifecycle` as queryable workflow history under the canonical issue/plan relationship.

## Context

The plan predates the requirement that every plan implements exactly one issue and had no recoverable primary issue link.

## Behaviour

The migrated plan has one `implements` relationship to this issue, so incoming graph edges expose its history without invented product requirements.

## Scope

Mechanical metadata and required schema sections only; historical intent and product behavior are unchanged.

## Resolution

The historical work is represented by `plan-011-filing-by-lifecycle`; this migration-derived issue preserves the required one-issue-per-plan relationship.
