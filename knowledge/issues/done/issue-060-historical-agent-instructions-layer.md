---
type: Issue
id: issue-060-historical-agent-instructions-layer
title: 'Historical plan: Agent Instructions Layer'
description: Migration-derived issue preserving the primary issue relationship for plan-002-agent-instructions-layer.
kind: chore
lifecycle: done
---

# Chore: historical plan: agent instructions layer

## Summary

Preserve `plan-002-agent-instructions-layer` as queryable workflow history under the canonical issue/plan relationship.

## Context

The plan predates the requirement that every plan implements exactly one issue and had no recoverable primary issue link.

## Behaviour

The migrated plan has one `implements` relationship to this issue, so incoming graph edges expose its history without invented product requirements.

## Scope

Mechanical metadata and required schema sections only; historical intent and product behavior are unchanged.

## Resolution

The historical work is represented by `plan-002-agent-instructions-layer`; this migration-derived issue preserves the required one-issue-per-plan relationship.
