---
type: Issue
id: issue-069-historical-contract-driven-workflow
title: 'Historical plan: The workflow becomes contract-driven'
description: Migration-derived issue preserving the primary issue relationship for plan-012-contract-driven-workflow.
kind: chore
lifecycle: done
---

# Chore: historical plan: the workflow becomes contract-driven

## Summary

Preserve `plan-012-contract-driven-workflow` as queryable workflow history under the canonical issue/plan relationship.

## Context

The plan predates the requirement that every plan implements exactly one issue and had no recoverable primary issue link.

## Behaviour

The migrated plan has one `implements` relationship to this issue, so incoming graph edges expose its history without invented product requirements.

## Scope

Mechanical metadata and required schema sections only; historical intent and product behavior are unchanged.

## Resolution

The historical work is represented by `plan-012-contract-driven-workflow`; this migration-derived issue preserves the required one-issue-per-plan relationship.
