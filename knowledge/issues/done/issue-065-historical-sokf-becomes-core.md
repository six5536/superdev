---
type: Issue
id: issue-065-historical-sokf-becomes-core
title: 'Historical plan: SOKF becomes a core part of superdev'
description: Migration-derived issue preserving the primary issue relationship for plan-008-sokf-becomes-core.
kind: chore
lifecycle: done
---

# Chore: historical plan: sokf becomes a core part of superdev

## Summary

Preserve `plan-008-sokf-becomes-core` as queryable workflow history under the canonical issue/plan relationship.

## Context

The plan predates the requirement that every plan implements exactly one issue and had no recoverable primary issue link.

## Behaviour

The migrated plan has one `implements` relationship to this issue, so incoming graph edges expose its history without invented product requirements.

## Scope

Mechanical metadata and required schema sections only; historical intent and product behavior are unchanged.

## Resolution

The historical work is represented by `plan-008-sokf-becomes-core`; this migration-derived issue preserves the required one-issue-per-plan relationship.
