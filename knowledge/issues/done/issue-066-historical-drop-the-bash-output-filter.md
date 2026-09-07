---
type: Issue
id: issue-066-historical-drop-the-bash-output-filter
title: 'Historical plan: Drop rtk and the bash-output-filter capability'
description: Migration-derived issue preserving the primary issue relationship for plan-009-drop-the-bash-output-filter.
kind: chore
lifecycle: done
---

# Chore: historical plan: drop rtk and the bash-output-filter capability

## Summary

Preserve `plan-009-drop-the-bash-output-filter` as queryable workflow history under the canonical issue/plan relationship.

## Context

The plan predates the requirement that every plan implements exactly one issue and had no recoverable primary issue link.

## Behaviour

The migrated plan has one `implements` relationship to this issue, so incoming graph edges expose its history without invented product requirements.

## Scope

Mechanical metadata and required schema sections only; historical intent and product behavior are unchanged.

## Resolution

The historical work is represented by `plan-009-drop-the-bash-output-filter`; this migration-derived issue preserves the required one-issue-per-plan relationship.
