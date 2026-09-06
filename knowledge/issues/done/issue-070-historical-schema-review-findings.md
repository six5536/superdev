---
type: Issue
id: issue-070-historical-schema-review-findings
title: 'Historical plan: Bring every schema in line with its own rules and the workflow'
description: Migration-derived issue preserving the primary issue relationship for plan-014-schema-review-findings.
kind: chore
lifecycle: done
---

# Chore: historical plan: bring every schema in line with its own rules and the workflow

## Summary

Preserve `plan-014-schema-review-findings` as queryable workflow history under the canonical issue/plan relationship.

## Context

The plan predates the requirement that every plan implements exactly one issue and had no recoverable primary issue link.

## Behaviour

The migrated plan has one `implements` relationship to this issue, so incoming graph edges expose its history without invented product requirements.

## Scope

Mechanical metadata and required schema sections only; historical intent and product behavior are unchanged.

## Resolution

The historical work is represented by `plan-014-schema-review-findings`; this migration-derived issue preserves the required one-issue-per-plan relationship.
